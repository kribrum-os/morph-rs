Все контрольные замеры проводились на машине с процессором AMD Ryzen™ 9 7900X × 24 и RAM-памятью 64,0 GiB.

# Pymorphy 

## Pymorphy3::parse(), Pymorphy3::normalize() на словах Войны и мир

Pymorphy3. War&Peace/parse
- time:   [1.3534 s **1.4194 s** 1.5007 s]
- thrpt:  [406.37 KiB/s **429.65 KiB/s** 450.60 KiB/s]

Pymorphy3. War&Peace/normalize
- time:   [1.0550 s **1.1703 s** 1.3208 s]
- thrpt:  thrpt:  [461.74 KiB/s **521.10 KiB/s** 578.07 KiB/s]

# Release 0.1.0
## MorphAnalyzer::init(..):
10 samples

- time:   [29.340 s **29.613 s** 29.867 s]
- thrpt:  [13.435 MiB/s **13.550 MiB/s** 13.676 MiB/s]

## MorphAnalyzer::parse(), MorphAnalyzer::normalize() для словарных слов на словах Войны и мир

War&Peace/parse/0
- time:   [16.091 ms **16.174 ms** 16.276 ms]
- thrpt:  [38.792 MiB/s **39.038 MiB/s** 39.238 MiB/s]

War&Peace/normalize/1
- thrpt:  [28.646 ms **28.842 ms** 29.068 ms]
- thrpt:  [21.721 MiB/s **21.891 MiB/s** 22.041 MiB/s]

# Release 0.2.1

## MorphAnalyzer::init(..):
10 samples

Mops init/init/0        
- time:   [31.744 s **32.236 s** 32.762 s]
- thrpt:  [12.248 MiB/s **12.447 MiB/s** 12.640 MiB/s]

## MorphAnalyzer::parse(), MorphAnalyzer::normalize() для словарных слов и префиксного вангования на словах Войны и мир

War&Peace words. Dictionary + Prefix Vanga/parse/0
- time:   [18.765 ms **19.267 ms** 19.822 ms]
- thrpt:  [31.852 MiB/s **32.770 MiB/s** 33.648 MiB/s]

War&Peace words. Dictionary + Prefix Vanga/normalize/1
- time:   [38.619 ms **39.268 ms** 39.949 ms]
- thrpt:  [15.805 MiB/s **16.079 MiB/s** 16.349 MiB/s]

## MorphAnalyzer::declension()
100 words of War&Peace, 10 samples

Declension 1000 dictionary words/declension/0
- time:   [4.2209 s **4.3062 s** 4.4033 s]
- thrpt:  [146.83 KiB/s **150.14 KiB/s** 153.17 KiB/s]

## MorphAnalyzer::inflect() для всех тестовых слова
10 samples 

Inflect Words/inflection/0
- time:   [35.080 s **37.304 s** 39.824 s]
- thrpt:  [173.91   B/s **185.66   B/s** 197.43   B/s]
