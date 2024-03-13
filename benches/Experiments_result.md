# Release 0.2.0. Эксперименты.

## Образование словаря со сливанием лемм для нормализации.

### I Образование словаря с нормализацией, где алгоритм сочленения лемм происходит от лемм к связям.

```rust
        for lemma in lemmata.lemmas.iter_mut() {
             let inizio = SmallString::from_str(&lemma.normal_form.text);

             let link = links
                 .links
                 .iter()
                 .find(|link| link.lemma_id == lemma.id || link.variant == lemma.id);
             // Если лемма выступает в каких-либо отношениях...
             if let Some(link) = link {
                 let Link {
                     type_id,
                     lemma_id,
                     variant,
                 } = link;
                 // Если это не последний проход к нормальной форме..
                 if DOUBLE_FROM.contains(type_id) {
                     let real_lemma = links
                         .links
                         .iter()
                         .find(|link| link.variant == *lemma_id)
                         .unwrap()
                         .lemma_id;
                    <..>
                 // Если мы пришли к нормальной форме
                 } else {
                     // Лемма - нормальная форма
                     if lemma.id == *lemma_id {
                         lemmas.push(inizio);

                         <..>
                     }
                     // Лемма - это побочная форма, но мы ее сохраняем как начальную
                     else if lemma.id == *variant {
                         <..>
                     }
                 }
             // Если лемма не выступает в каких-либо отношениях
             } else {
                 lemmas.push(inizio);
                 <..>
             }
         }
```

- time:   [75.536 s **75.991 s** 76.478 sN]
- thrpt:  [5.2466 MiB/s **5.2803 MiB/s** 5.3120 MiB/s]

### Образование словаря с нормализацией, где алгоритм сначала сочленяет леммы, а потом собирает их

```rust

    let link_connotation: HashMap<u64, Vec<u64>> = links.clone().collect_lemmas();

    for (lemma_id, variants) in link_connotation {
            let normal = lemmata.iter().find(|l| l.id == lemma_id).unwrap();
            let normal_form = normal.normal_form.text;

            lemmas.push(SmallString::from_str(&normal_form));

            <..>

            for lemma in variants {
                let lemma = lemmata.iter().find(|l| l.id == lemma).unwrap();
                <..>
            }
        }
```

- time:   [64.938 s **66.711 s** 68.035 s]с
- thrpt:  [5.8977 MiB/s **6.0148 MiB/s** 6.1790 MiB/s]

### Оптимизация предыдущего подхода.

Вместо постоянной итерации с поиском по номеру леммы, мы превращаем
вектор лемм в `HashMap<LemmaId, LemmaDict { normal_form: _, variants: _}>`. Это позволяет находить LemmaDict по ключу.

Хотелось бы провернуть поиск по индексу в векторе лемм, чтобы индекс совпадал с id, но, к сожалению, словарь OpenCorpora
прописывает леммы не подряд.

```rust
let mut lemmata_map = HashMap::new();
        for lemma in lemmata.lemmas {
            lemmata_map.insert(
                lemma.id,
                LemmaDict {
                    normal_form: lemma.normal_form,
                    variants: lemma.forms,
                },
            );
        }
```

- time:   [30.896 s **30.989 s** 31.083 s]
- thrpt:  [12.909 MiB/s **12.948 MiB/s** 12.987 MiB/s]

## Буква ё. 
Замеры проводились под нестабилизированным вангованием, поэтому могут являться не идеальными замерами для словаря в целом.

### Буква ё сохраняется только как ё.

- time:   [31.301 s **31.455 s** 31.612 s]
- thrpt:  [12.693 MiB/s **12.756 MiB/s** 12.819 MiB/s]

Размер fst при этом: 383164 байт

### Слова с буквой ё дублируется, все ё в них заменяются на букву е.

- time:   [31.753 s **32.005 s** 32.256 s]
- thrpt:  [12.439 MiB/s **12.537 MiB/s** 12.637 MiB/s]

Размер fst при этом: 398456 байт

## Inflect/declension - поиск не по всему fst, а по префиксу слова.

Многие слова во всех своих склонениях/пряжениях начинаются с одной и той же буквы. 
Например, `ставшего, ставшее, ставшей, ставшем, ставшему, ставшею, ставши, ставшие, ставший, ставшим, ставшими`.
Нам не нужно при этом смотреть весь `fst`-словарь через `Stream` (чья производительность очень низкая).
Мы можем взять первые 1-2 буквы и смотреть в диапазоне этих двух букв.

Некоторые слова при этом чередуются по буквам. Для таких был составлен список diff-ов: `src/analyzer/declension.rs -> DIFF_FORM`.

Для тестов на `declension` прогонялись первые 1000 слов из датасета со словами из Войны и мир (`data/words.txt`).
Для тестов на `inflect` из 1000 слов Войны и мир были взяты все причастия, которые в тесте приводились к глаголу женского рода (`data/inflect.txt`).

Брались сравнительные тесты для одной буквы в префиксе и для двух букв в префиксе.
Для трех уже был слишком большой diff, чтобы можно было хранить его в константе.

### Rust 0.2.0. Declension 1000 words/declension/one char for fst
- time:   [130.96 s **131.93 s** 132.79 s]
- thrpt:  [[4.8689 KiB/s **4.9008 KiB/s** 4.9369 KiB/s]

### Rust 0.2.0. Declension 1000 words/declension/two chars for fst
- time:   [122.32 s **122.55 s** 122.77 s]
- thrpt:  [[5.2665 KiB/s **5.2756 KiB/s** 5.2858 KiB/s]

### Rust 0.2.0. Inflection All Words/inflection/one char for fst
- time:   [114.76 s **115.46 s** 116.67 s]
- thrpt:  [[5.5414 KiB/s **5.5997 KiB/s** 5.6337 KiB/s]

### Rust 0.2.0. Inflection All Words/inflection/two chars for fst
- time:   [106.39 s **106.94 s** 107.47 s]
- thrpt:  [[6.0158 KiB/s **6.0459 KiB/s** 6.0773 KiB/s]
