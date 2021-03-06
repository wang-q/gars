SELECT `distance`,
       round(avg(gc_content), 4)  AVG_gc_content,
       round(avg(gc_mean), 4)     AVG_gc_mean,
       round(avg(gc_stddev), 4)   AVG_gc_stddev,
       round(avg(gc_cv), 4)       AVG_gc_cv,
       round(avg(cdsProp), 4)     AVG_cdsProp,
       round(avg(repeatsProp), 4) AVG_repeatsProp,
       count(ID)                  COUNT
FROM fsw
GROUP BY distance
ORDER BY distance
    FORMAT TSVWithNames
