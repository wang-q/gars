# Atha

## genome

```shell script
mkdir -p ~/data/garr/Atha/genome
cd ~/data/garr/Atha/genome

# download
aria2c -j 4 -x 4 -s 2 -c --file-allocation=none \
    http://ftp.ensemblgenomes.org/pub/release-45/plants/fasta/arabidopsis_thaliana/dna/Arabidopsis_thaliana.TAIR10.dna_sm.toplevel.fa.gz

# chromosomes
gzip -d -c *dna_sm.toplevel* > toplevel.fa

faops count toplevel.fa |
    perl -nla -e '
        next if $F[0] eq 'total';
        print $F[0] if $F[1] > 50000;
        print $F[0] if $F[1] > 5000  and $F[6]/$F[1] < 0.05;
    ' |
    uniq > listFile
faops some toplevel.fa listFile stdout |
    faops filter -N stdin stdout |
    faops split-name stdin .
rm toplevel.fa listFile

mv Mt.fa Mt.fa.skip
mv Pt.fa Pt.fa.skip

# .fa.gz
cat {1..5}.fa |
    gzip -9 \
    > genome.fa.gz
faops size genome.fa.gz > chr.sizes

```

## T-DNA

```shell script
mkdir -p ~/data/garr/Atha/features/
cd ~/data/garr/Atha/features/

for name in CSHL FLAG MX RATM; do
    aria2c -j 4 -x 4 -s 2 --file-allocation=none -c \
        http://natural.salk.edu/database/tdnaexpress/T-DNA.${name}
done

# Convert to positions
for name in CSHL FLAG MX RATM; do
    cat T-DNA.${name} |
         perl -nla -e '
            @F >= 2 or next;
            next unless $F[1];

            my ( $chr, $pos ) = split /:/, $F[1];
            $chr =~ s/chr0?//i;
            $pos =~ s/^0+//;
            next unless $chr =~ /^\d+$/;

            print "$chr:$pos";
        ' \
        > T-DNA.${name}.pos.txt;
done

```

## `garr`

### Contigs

```shell script
# start redis-server
redis-server --appendonly no --dir ~/data/garr/Atha/

cd ~/data/garr/Atha/

garr env --all

garr status drop

garr gen genome/genome.fa.gz --piece 500000

# redis dumps
mkdir -p ~/data/garr/Atha/dumps/

while true; do
    garr status dump
    if [ $? -eq 0 ]; then
        cp dump.rdb dumps/ctg.dump.rdb
        break
    fi
    sleep 5
done

# tsv exports
mkdir -p tsvs

garr tsv -s 'ctg:*' -f length | head

garr tsv -s 'ctg:*' |
    keep-header -- tsv-sort -k2,2 -k3,3n -k4,4n \
    > tsvs/ctg.tsv

cat tsvs/ctg.tsv |
    sed '1d' |
    cut -f 1 \
    > ctg.lst

# positions
parallel -j 4 -k --line-buffer '
    echo {}
    garr pos features/T-DNA.{}.pos.txt
    ' ::: CSHL FLAG MX RATM

garr tsv -s 'pos:*' |
    keep-header -- tsv-sort -k2,2 -k3,3n -k4,4n \
    > tsvs/pos.tsv

# dumps
while true; do
    garr status dump
    if [ $? -eq 0 ]; then
        mkdir -p dumps
        cp dump.rdb dumps/pos.dump.rdb
        break
    fi
    sleep 5
done

# stop the server
garr status stop

sudo /etc/init.d/clickhouse-server start

```

### Ranges and rsw

Benchmarks keydb against redis

```shell script
cd ~/data/garr/Atha/

rm ./dump.rdb

redis-server --appendonly no --dir ~/data/garr/Atha/
#keydb-server --appendonly no --dir ~/data/garr/Atha/
# keydb is as fast/slow as redis

garr env

garr status drop

time garr gen genome/genome.fa.gz --piece 500000
#real    0m1.520s
#user    0m0.582s
#sys     0m0.407s

time parallel -j 4 -k --line-buffer '
    echo {}
    garr range features/T-DNA.{}.pos.txt --tag {}
    ' ::: CSHL # FLAG MX RATM
# redis
# RATM
#real    0m14.055s
#user    0m1.819s
#sys     0m6.357s
# 4 files
#real    0m40.654s
#user    0m11.503s
#sys     0m21.387s

# keydb
# RATM
#real    0m14.228s
#user    0m1.792s
#sys     0m6.314s
# 4 files
#real    0m42.186s
#user    0m11.481s
#sys     0m21.391s

garr tsv -s "range:*" |
    keep-header -- tsv-sort -k2,2 -k3,3n -k4,4n \
    > tsvs/range.tsv

while true; do
    garr status dump
    if [ $? -eq 0 ]; then
        mkdir -p dumps
        cp dump.rdb dumps/range.dump.rdb
        break
    fi
    sleep 5
done

# rsw
time cat ctg.lst |
    parallel -j 4 -k --line-buffer '
        garr rsw --ctg {}
        ' |
    tsv-uniq |
    keep-header -- tsv-sort -k2,2 -k3,3n -k4,4n \
    > tsvs/rsw.tsv
# CSHL
# -j 4
#real    7m43.384s
#user    24m58.916s
#sys     3m42.415s
# -j 2
#real    13m38.805s
#user    21m56.417s
#sys     4m34.154s

garr status stop

```

### GC-wave

Restores from ctg.dump.rdb

```shell script
cd ~/data/garr/Atha/

cp dumps/ctg.dump.rdb ./dump.rdb

redis-server --appendonly no --dir ~/data/garr/Atha/ &

garr env

time cat ctg.lst |
    parallel -j 4 -k --line-buffer '
        garr sliding \
            --ctg {} \
            --size 100 --step 1 \
            --lag 1000 \
            --threshold 3.0 \
            --influence 1.0 \
            -o stdout |
            tsv-filter -H --ne signal:0 \
            > {.}.gc.tsv

        cat {.}.gc.tsv |
            cut -f 1 |
            linkr merge -c 0.8 stdin -o {.}.replace.tsv

        cat {.}.gc.tsv |
            ovlpr replace stdin {.}.replace.tsv |
            tsv-uniq -H -f 1 \
            > tsvs/{.}.peak.tsv

        tsv-summarize tsvs/{.}.peak.tsv \
            -H --group-by signal --count

        rm {.}.gc.tsv {.}.replace.tsv
    '
#real    5m58.731s
#user    23m47.703s
#sys     0m16.114s

# Don't need to be sorted
tsv-append -f <(cat ctg.lst | sed 's/$/.peak.tsv/') -H \
    > tsvs/peak.tsv
rm tsvs/ctg:*.peak.tsv

tsv-summarize tsvs/peak.tsv \
    -H --group-by signal --count
#signal  count
#1       32211
#-1      26821

time garr wave tsvs/peak.tsv
#real    4m27.902s
#user    0m26.255s
#sys     2m31.036s

garr tsv -s "peak:*" |
    keep-header -- tsv-sort -k2,2 -k3,3n -k4,4n \
    > tsvs/wave.tsv

cat tsvs/wave.tsv |
    tsv-summarize -H --count
# 59032

tsv-filter tsvs/wave.tsv -H --or \
    --le left_wave_length:0 --le right_wave_length:0 |
    tsv-summarize -H --count
# 12927

```

## Benchmarks

```shell script
redis-server --appendonly no --dir ~/data/garr/Atha/

cd ~/data/garr/Atha/

garr env

hyperfine --warmup 1 --export-markdown garr.md.tmp \
    '
        garr status drop;
        garr gen genome/genome.fa.gz --piece 500000;
    ' \
    '
        garr status drop;
        garr gen genome/genome.fa.gz --piece 500000;
        garr pos features/T-DNA.CSHL.pos.txt;
    ' \
    '
        garr status drop;
        garr gen genome/genome.fa.gz --piece 500000;
        garr range features/T-DNA.CSHL.pos.txt --tag CSHL;
    ' \
    '
        garr status drop;
        garr gen genome/genome.fa.gz --piece 500000;
        garr sliding --size 100 --step 20 --lag 50 |
            tsv-filter -H --ne signal:0 > /dev/null;
    '

cat garr.md.tmp

```

| Command               |       Mean [ms] | Min [ms] | Max [ms] |     Relative |
|:----------------------|----------------:|---------:|---------:|-------------:|
| `drop; gen;`          |    813.8 ± 18.5 |    783.4 |    834.4 |         1.00 |
| `drop; gen; pos;`     |  8203.5 ± 166.7 |   8051.7 |   8475.7 | 10.08 ± 0.31 |
| `drop; gen; range;`   | 20045.7 ± 474.6 |  19218.6 |  21041.7 | 24.63 ± 0.81 |
| `drop; gen; sliding;` |   7580.7 ± 72.6 |   7467.1 |   7705.8 |  9.31 ± 0.23 |

## clickhouse

* server

```shell script
cd ~/data/garr/Atha/

mkdir -p clickhouse
cd clickhouse
clickhouse server

```

* queries

```shell script
cd ~/data/garr/Atha/

clickhouse client --query "$(cat sqls/ddl/ctg.sql)"
cat tsvs/ctg.tsv |
    clickhouse client --query "INSERT INTO ctg FORMAT TSVWithNames"

```