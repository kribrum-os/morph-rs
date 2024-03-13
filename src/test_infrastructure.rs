#[cfg(test)]
pub(crate) mod infrastructure {
    use crate::{
        analyzer::{Dictionary, Tag},
        morph::grammemes::{Grammem, ParteSpeech},
        opencorpora::DictionaryOpenCorpora,
        Language,
    };
    use std::{collections::HashMap, path::PathBuf};

    /// Создание тестового словаря + fst для проверки функций.
    /// Имя файла принимается без формата `.xml`.
    pub(crate) fn make_dict(file_path: &str, fst: PathBuf) -> Dictionary {
        let dict = DictionaryOpenCorpora::init_from_path(file_path)
            .unwrap_or_else(|_| panic!("File: {file_path}"));
        Dictionary::from_opencorpora(dict, fst, Language::Russian)
            .map_err(|err| panic!("Dictionary Err: {err:?}"))
            .unwrap()
    }

    pub(crate) fn is_diff(
        chars_diff: &mut HashMap<String, Vec<String>>,
        chars: usize,
        inizio_chars: &str,
        normal_form: &str,
        word: &str,
        grammemes: Tag,
    ) {
        let inizio_chars = inizio_chars.replace('ё', "е");
        let normal_form = normal_form.replace('ё', "е");
        let word = word.replace('ё', "е");
        if !word.starts_with(&inizio_chars)
            && Grammem::pos_in_tag(&grammemes).unwrap() != ParteSpeech::Comparative
            && Grammem::pos_in_tag(&grammemes).unwrap() != ParteSpeech::AdjectiveFull
            && Grammem::pos_in_tag(&grammemes).unwrap() != ParteSpeech::AdjectiveShort
            && Grammem::pos_in_tag(&grammemes).unwrap() != ParteSpeech::Adverb
        {
            let another_chars = word.chars().take(chars).collect::<String>();

            let diff = chars_diff.entry(normal_form).or_default();
            if !diff.contains(&another_chars) {
                diff.push(another_chars);
            }
        };
    }
}

#[cfg(test)]
mod compile_const {
    use crate::{analyzer::dictionary::LemmaDict, opencorpora::DictionaryOpenCorpora};
    use std::{collections::HashMap, io::Write};
    use test_case::test_case;

    #[ignore = "Too large dictionary. Manual test for each new lemma's links"]
    #[test_case(1; "1_prefix_chars")]
    #[test_case(2; "2_prefix_chars")]
    #[test_case(3; "3_prefix_chars")]
    /// Для некоторых нормализованных форм характерно чередование букв в префиксе.
    /// Для того, чтобы найти все слова для спряжений/склонений, нам необходимо
    /// проверить также префиксы с чередованием.
    ///
    /// В данной функции мы собираем пары "нормальная форма - префикс с чередованием".
    fn test_form_first_chars(chars: usize) {
        let mut chars_diff: HashMap<String, Vec<String>> = HashMap::new();

        let dict = DictionaryOpenCorpora::init_from_path("dict.opcorpora.xml").unwrap();

        let link_connotation: HashMap<u64, Vec<u64>> = dict.links.clone().collect_lemmas();

        let mut all_lemmas = std::collections::BTreeSet::new();

        let mut lemmata_map = HashMap::new();
        for lemma in dict.lemmata.lemmas {
            all_lemmas.insert(lemma.id);
            lemmata_map.insert(
                lemma.id,
                LemmaDict {
                    normal_form: lemma.normal_form,
                    variants: lemma.forms,
                },
            );
        }

        for (lemma_id, variants) in link_connotation {
            let normal_lemma = &lemmata_map.get(&lemma_id).unwrap();

            let normal_form = normal_lemma.normal_form.text.clone();
            let inizio_chars = normal_form.chars().take(chars).collect::<String>();

            let inizio_grammemes = normal_lemma.first_tags().unwrap();

            match normal_lemma.variants.as_ref().filter(|v| !v.is_empty()) {
                None => {}
                Some(forms) => {
                    let mut iter = LemmaDict::forms(forms.to_owned().to_owned());

                    let (first_word, mut first_grammemes) = iter.next().unwrap();
                    first_grammemes.extend(inizio_grammemes.clone());
                    crate::test_infrastructure::infrastructure::is_diff(
                        &mut chars_diff,
                        chars,
                        &inizio_chars,
                        &normal_form,
                        &first_word,
                        first_grammemes,
                    );

                    for (word, mut grammemes) in iter {
                        grammemes.extend(inizio_grammemes.clone());
                        crate::test_infrastructure::infrastructure::is_diff(
                            &mut chars_diff,
                            chars,
                            &inizio_chars,
                            &normal_form,
                            &word,
                            grammemes,
                        )
                    }
                }
            }

            all_lemmas.remove(&lemma_id);

            for variant_id in variants {
                let lemma = lemmata_map.get(&variant_id).unwrap();

                match lemma.variants.as_ref().filter(|v| !v.is_empty()) {
                    None => {}
                    Some(forms) => {
                        let mut iter = LemmaDict::forms(forms.to_owned().to_owned());

                        let (first_word, mut first_grammemes) = iter.next().unwrap();

                        first_grammemes.extend(inizio_grammemes.clone());
                        crate::test_infrastructure::infrastructure::is_diff(
                            &mut chars_diff,
                            chars,
                            &inizio_chars,
                            &normal_form,
                            &first_word,
                            first_grammemes,
                        );

                        for (word, mut grammemes) in iter {
                            grammemes.extend(inizio_grammemes.clone());
                            crate::test_infrastructure::infrastructure::is_diff(
                                &mut chars_diff,
                                chars,
                                &inizio_chars,
                                &normal_form,
                                &word,
                                grammemes,
                            )
                        }
                    }
                }
                all_lemmas.remove(&variant_id);
            }
        }

        // Осташиеся не участвующие в link_connotations леммы также нужно обработать.
        for lost_id in all_lemmas {
            let lemma = lemmata_map.get(&lost_id).unwrap();
            let normal_form = lemma.normal_form.text.clone();

            let inizio_chars = normal_form.chars().take(chars).collect::<String>();

            let inizio_grammemes = lemma.first_tags().unwrap();

            match lemma.variants.as_ref().filter(|v| !v.is_empty()) {
                None => {}
                Some(forms) => {
                    let mut iter = LemmaDict::forms(forms.to_owned().to_owned());

                    let (first_word, mut first_grammemes) = iter.next().unwrap();
                    first_grammemes.extend(inizio_grammemes.clone());
                    crate::test_infrastructure::infrastructure::is_diff(
                        &mut chars_diff,
                        chars,
                        &inizio_chars,
                        &normal_form,
                        &first_word,
                        first_grammemes,
                    );

                    for (word, mut grammemes) in iter {
                        grammemes.extend(inizio_grammemes.clone());
                        crate::test_infrastructure::infrastructure::is_diff(
                            &mut chars_diff,
                            chars,
                            &inizio_chars,
                            &normal_form,
                            &word,
                            grammemes,
                        )
                    }
                }
            }
        }

        eprintln!("For {chars} chars diff_len is {}", chars_diff.len());

        let file = &format!("different_inizio_{chars}");
        let mut file = std::fs::File::create(file).unwrap();
        for (k, v) in chars_diff {
            for val in v {
                file.write_all(format!("(\"{k}\", \"{val}\"),\n").as_bytes())
                    .unwrap();
            }
        }
    }
}

#[cfg(test)]
mod experiments {
    use super::infrastructure::*;
    use crate::{
        morph::grammemes::{Number, ParteSpeech},
        opencorpora::{
            dictionary::{Lemma, Link},
            DictionaryOpenCorpora,
        },
        MorphAnalyzer, SMALLLEMMA, SMALLTAG, SMALLVANGA,
    };
    use std::io::Write;
    use tempfile::tempdir;

    #[ignore = "Manual test. Needed to run when serialization change"]
    #[test]
    /// Функция, которая проверяет, что все данные при десериализации сходятся с данными при инициализации.
    /// К сожалению, для `bincode` это не является корректным, поэтому мы не можем его использовать (по крайней мере пока).
    fn correct_serialization() {
        let dict_path = "dict.opcorpora.xml";
        let output = "data/result/test/";

        let mops = MorphAnalyzer::create(dict_path, output, crate::Language::Russian)
            .expect("Mops creation");
        let mops = MorphAnalyzer::init(mops, output).expect("Mops initialization");

        let mops2 = MorphAnalyzer::open(output).expect("Mops open");

        // Все уникальные слова из Войны и мир
        let binding = std::fs::read_to_string("benches/data/words.txt").expect("Read text file");
        let words = binding.lines();

        for word in words.clone() {
            let parse = mops.parse(word).map(|mut s| {
                s.0.sort();
                s
            });
            let parse2 = mops2.parse(word).map(|mut s| {
                s.0.sort();
                s
            });
            match (&parse, &parse2) {
                (Ok(p), Ok(p2)) => assert_eq!(p, p2),
                (Err(e), Err(e2)) => assert_eq!(e.to_string(), e2.to_string()),
                _ => {
                    tracing::error!("Diff: {parse:?}, {parse2:?}");
                    continue;
                }
            }
        }
    }

    #[ignore = "Manual test"]
    #[test]
    /// Тест для проверки слов, использующихся в NER-модели (Наташа).
    fn test_ner() {
        let anal = MorphAnalyzer::open("data/result/").unwrap();

        // Слова для NER из habr-статьи о Наташе.
        const WORDS_FOR_NER: [&str; 36] = [
            "время",
            "июле",
            "Анне",
            "Павловне",
            "Шерер",
            "Марии",
            "Федоровны",
            "Василия",
            "Инженерной",
            "улице",
            "Алтуфьевском",
            "шоссе",
            "республике",
            "Карелии",
            "телефон",
            "улице",
            "Маршала",
            "Мерецкова",
            "улицей",
            "Крузенштерна",
            "руб",
            "рубля",
            "рублей",
            "рублями",
            "января",
            "феврале",
            "марте",
            "апрелем",
            "маем",
            "июня",
            "июля",
            "августа",
            "сентября",
            "октябрь",
            "ноябрь",
            "декабрь",
        ];

        for word in WORDS_FOR_NER {
            let word = word.to_lowercase();
            if !anal.is_known(&word) {
                eprintln!("{word}")
            } else {
                eprintln!(
                    "Normal form: {}",
                    anal.normalize_get(&word, 0).unwrap().unwrap().word()
                )
            }
        }
    }

    #[test]
    /// Сбор причастий во мн.ч. из войны и мир для тестов на `inflect()` в бенчмарках.
    fn test_bench_inflect() {
        let anal = MorphAnalyzer::open("data/result/").unwrap();

        let binding = std::fs::read_to_string("benches/data/words.txt").expect("Read text file");
        let words = binding.lines().take(10000);

        let mut inflect = std::fs::File::create("benches/data/inflect.txt").unwrap();

        for word in words {
            let parse = anal.parse(word).unwrap();
            if let Some(w) = parse.find(grams![ParteSpeech::ParticipleFull, Number::Plural]) {
                let str = format!("{}\n", w.word());
                inflect.write_all(str.as_bytes()).unwrap();
            } else {
                continue;
            }
        }
    }

    #[ignore = "Too large dictionary. Needed to run when you change dictionary"]
    #[test]
    #[should_panic]
    /// Тест, показывающий, что у нас не все леммы участвуют в связях между леммами.
    /// Это требует от нас отдельного вычленения \
    /// единичных ~~сильных и независимых~~ лемм в предварительном парсинге.
    fn test_find_lemma_without_link() {
        let dict = DictionaryOpenCorpora::init_from_path("dict.opcorpora.xml").unwrap();

        let mut all_lemmas = std::collections::HashSet::new();

        for lemma in dict.lemmata.lemmas {
            all_lemmas.insert(lemma.id);
        }

        let mut link_lemmas = std::collections::HashSet::new();

        for Link {
            lemma_id, variant, ..
        } in dict.links.links
        {
            link_lemmas.insert(lemma_id);
            link_lemmas.insert(variant);
        }

        if all_lemmas != link_lemmas {
            panic!("Not common lemmas")
        }
    }

    #[ignore = "Too large dictionary. Experiment"]
    #[test]
    /// Тест, который выявит, есть ли у нас нормальные формы, служащие вариантом для других форм.
    fn try_double_normalize() {
        let dict = DictionaryOpenCorpora::init_from_path("dict.opcorpora.xml").unwrap();

        assert!(dict.links.links.iter().any(|Link { lemma_id, .. }| dict
            .links
            .links
            .iter()
            .any(|link| link.variant == *lemma_id)))
    }

    #[ignore = "Too large dictionary. Needed to run when you change dictionary"]
    #[should_panic]
    #[test]
    /// Тест на то, что все id леммы идут поочередно.
    /// Такого нет в изначальном словаре ОпенКорпоры, поэтому если не произойдет паники, значит мы улучшили словарь
    /// и можно оптимизировать процесс предобработки словаря.
    fn test_id_eq_position() {
        let dict = DictionaryOpenCorpora::init_from_path("dict.opcorpora.xml").unwrap();

        let mut lemmata = dict.lemmata.lemmas;
        lemmata.sort_by(|l, k| l.id.cmp(&k.id));

        for (i, Lemma { id, .. }) in lemmata.iter().enumerate() {
            assert_eq!(i as u64, *id - 1);
        }
    }

    #[ignore = "Too large dictionary"]
    #[test]
    /// Нахождение оптимального количества байтов для хранения Lemma через девяностый процентиль.
    /// TODO: Проверить после устоявшейся схемы нормализации.
    fn test_lemma_bytes() {
        let tmp_dir = tempdir().unwrap();
        let fst = tmp_dir.path().join("lemma_bytes.fst");

        let dict = make_dict("dict.opcorpora.xml", fst);

        let mut lengths = Vec::new();

        for lemma in dict.lemmas {
            let len = lemma.len();

            lengths.push(len);
        }

        lengths.sort();

        let ninety = lengths.get((lengths.len() as f32 * 0.9) as usize).unwrap();
        assert_eq!(SMALLLEMMA, *ninety);
    }

    #[ignore = "too large dictionary"]
    #[test]
    /// Нахождение оптимального количества байтов для хранения Tag через девяностый процентиль.
    /// TODO: Проверить после устоявшейся схемы нормализации.
    fn test_tag_bytes() {
        let tmp_dir = tempdir().unwrap();
        let fst = tmp_dir.path().join("tag_bytes.fst");

        let dict = make_dict("dict.opcorpora.xml", fst);

        let mut lengths = Vec::new();
        for tag in dict.tags {
            let len = tag.len();
            lengths.push(len);
        }
        lengths.sort();

        let ninety = lengths.get((lengths.len() as f32 * 0.9) as usize).unwrap();
        assert_eq!(SMALLTAG, *ninety);
    }

    #[ignore = "too large dictionary"]
    #[test]
    /// Нахождение оптимального количества байтов для хранения постфикса Vanga через девяностый процентиль.
    /// TODO: Проверить после устоявшейся схемы нормализации.
    fn test_vanga_bytes() {
        let tmp_dir = tempdir().unwrap();
        let fst = tmp_dir.path().join("vanga_bytes.fst");

        let dict = make_dict("dict.opcorpora.xml", fst);

        let mut lengths = Vec::new();
        let postfix = dict
            .paradigms
            .into_iter()
            .flat_map(|v| v.postfix.into_iter().map(|i| i.postfix));
        for suff in postfix {
            let len = suff.len();
            lengths.push(len);
        }
        lengths.sort();

        let ninety = lengths.get((lengths.len() as f32 * 0.9) as usize).unwrap();
        assert_eq!(SMALLVANGA, *ninety);
    }
}
