Все контрольные замеры проводились на машине с процессором AMD Ryzen™ 9 7900X × 24 и RAM-памятью 64,0 GiB.

# Release 0.1.0
## MorphAnalyzer::init(..):
10 samples

- time:   [29.340 s **29.613 s** 29.867 s]
- thrpt:  [13.435 MiB/s **13.550 MiB/s** 13.676 MiB/s]

## MorphAnalyzer::parse(), MorphAnalyzer::normalize() для словарных слов на словах Войны и мир

Rust 0.1.0. Word&Peace/parse/0
- time:   [16.091 ms **16.174 ms** 16.276 ms]
- thrpt:  [38.792 MiB/s **39.038 MiB/s** 39.238 MiB/s]

Rust 0.1.0. Word&Peace/normalize/1
- thrpt:  [28.646 ms **28.842 ms** 29.068 ms]
- thrpt:  [21.721 MiB/s **21.891 MiB/s** 22.041 MiB/s]

## Pymorphy3::parse(), Pymorphy3::normalize() для словарных слов на словах Войны и мир

Pymorphy3. Word&Peace/parse
- time:   [1.3534 s **1.4194 s** 1.5007 s]
- thrpt:  [406.37 KiB/s **429.65 KiB/s** 450.60 KiB/s]

Pymorphy3. Word&Peace/normalize
- time:   [1.0550 s **1.1703 s** 1.3208 s]
- thrpt:  thrpt:  [461.74 KiB/s **521.10 KiB/s** 578.07 KiB/s]