CREATE TABLE ctg
(
    `ID`         String,
    `chr_id`     String,
    `chr_start`  UInt32,
    `chr_end`    UInt32,
    `chr_strand` String,
    `length`     UInt32
) ENGINE = MergeTree()
      PRIMARY KEY (`ID`)
      ORDER BY (`ID`, `chr_id`, `chr_start`, `chr_end`);
