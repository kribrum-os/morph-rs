use itertools::Itertools;
use serde::{Deserialize, Serialize};
use smallstr::SmallString;
use tracing::error;

use crate::{
    errors::{DictionaryErr, MopsResult},
    morph::grammemes::{FVanga, Form, Grammem, ParteSpeech},
    opencorpora::dictionary::Lemma,
    MorphAnalyzer, Vangovanie,
};

use super::{dictionary::longest_common_substring, Dictionary, Tag, Tags, Vanga, VangaItem};

#[allow(dead_code)]
/// Вычленение стемма несловарного слова по тому, какой ему был найден аффикс.
pub fn stemming(word: &str, affix: &str) -> String {
    let stem = if let Some(stem) = word.strip_prefix(affix) {
        stem
    } else if let Some(stem) = word.strip_suffix(affix) {
        stem
    } else {
        word
    };

    stem.to_string()
}

impl VangaItem {
    /// Сравнение на эквивалентность только по постфиксу.
    pub fn equal(&self, another: &Self) -> bool {
        self.postfix == another.postfix
    }

    /// Если две VangaItem равны, то первая насыщается тегами второй.
    pub fn saturation(&mut self, another: &Self) {
        if self.equal(another) {
            for vec_tag in &another.tag {
                self.tag.push(vec_tag.to_owned());
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Serialize, Deserialize)]
pub struct VangaItemIntermediate {
    pub(crate) postfix: String,
    pub(crate) form: Form,
    pub(crate) tag: Vec<Tag>,
}

impl std::fmt::Display for VangaItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.postfix)
    }
}

impl VangaItemIntermediate {
    /// Из набора форм леммы вычленяет общий корень и отсекает префикс и суффикс.
    pub fn parse_vanga_item(
        stem: &str,
        to_vanga_item: &str,
        mut tag: Tag,
        form: Form,
    ) -> Option<Self> {
        // Пока мы убираем ё.
        // TODO release 0.1.2
        let word = to_vanga_item
            .chars()
            .map(|mut char| {
                if char == 'ё' {
                    char = 'е';
                }
                char
            })
            .join("");

        tag.sort();

        match word.split_once(stem) {
            Some((_, postfix)) => {
                // По аналогии с Pymorphy2 убираем постфиксы более 5 символов.
                let count = postfix.chars().count();
                if count > 0 && count <= 5 {
                    Some(Self {
                        postfix: postfix.to_owned(),
                        form,
                        tag: vec![tag],
                    })
                } else {
                    None
                }
            }
            None => {
                todo!("error у нас не может не быть основы, мы ее специально вычленили")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Serialize, Deserialize)]
/// `Vanga` - предсказание по части речи на основе суффикса-префикса.
pub struct VangaIntermediate {
    pub(crate) popularity: u64,
    pub postfix: Vec<VangaItemIntermediate>,
}

impl Vanga {
    /// Преобразование предопределенных `VangaIntermediate` в конечную `Vanga`
    /// с `VangaItem` и `TagsID` для тегов.
    pub fn parse_vangas(
        stems: Vec<VangaIntermediate>,
        all_tags: &Tags,
    ) -> Result<Vec<Self>, DictionaryErr> {
        let mut vangas = Vec::new();

        for VangaIntermediate {
            popularity,
            postfix,
        } in stems
        {
            // По аналогии с Pymorphy2 убираем непродуктивные парадигмы, т.е. встреченные менее трех раз.
            // Todo сейчас все оказались уникальными
            if popularity < 3 {
                continue;
            } else {
                let mut items = Vec::new();

                for VangaItemIntermediate {
                    postfix,
                    form,
                    mut tag,
                } in postfix
                {
                    let mut tags = Vec::new();
                    tag.sort();

                    for tag in tag {
                        tags.push(
                            all_tags
                                .binary_search(&tag)
                                .map_err(|_| DictionaryErr::BinaryTag(tag.clone()))?,
                        )
                    }

                    let item = VangaItem {
                        postfix: SmallString::from(postfix),
                        form,
                        tag: tags,
                    };

                    items.push(item);
                }

                vangas.push(Vanga {
                    popularity,
                    postfix: items,
                });
            }
        }

        Ok(vangas
            .into_iter()
            .sorted_by(|a, b| b.popularity.cmp(&a.popularity))
            .collect_vec())
    }

    /// Сравнение на эквивалентность только по постфиксу всех внутренних `VangaItem`.
    pub fn equal(&self, another: &Self) -> bool {
        self.postfix
            .iter()
            .all(|a| another.postfix.iter().any(|b| a.equal(b)))
    }

    // TODO ААААА кто ж так алгоритмы делает
    pub fn saturation(&mut self, another: &Self) {
        // Если мы нашли дву одинаковые Ванги, то нам нужно насытить внутренние Item-сы тегами друг друга.
        if self.equal(another) {
            // eprintln!("одинаковые ванги найдены!!");
            for item in self.postfix.iter_mut() {
                for a_item in another.postfix.iter() {
                    item.saturation(a_item);
                }
            }
            // При этом если мы нашла такой же набор префиксов-постфиксов, то популярность Ванги возрастает на 1.
            self.popularity += 1;
        }
    }
}

impl Lemma {
    /// "Словарное слово", от которого может быть образовано другое слово через аффиксы, не может быть всеми частями речи (например, междометие не имеет аффиксов),
    /// поэтому мы определяем только те части речи, которые могут иметь приставку-суффикс.
    // todo release 0.1.1 после разделения нормализации и привидения к начальной форме слова, необходимо поправить
    const VANGA_POS: [ParteSpeech; 11] = [
        ParteSpeech::Noun,
        ParteSpeech::AdjectiveFull,
        ParteSpeech::Verb,           // todo при нормализации сведется к Infinitive
        ParteSpeech::ParticipleFull, // todo при нормализации сведется к Infinitive
        ParteSpeech::Gerundive,      // todo при нормализации сведется к Infinitive
        // Следующих слов нет в обозначенных по алгоритму поиска через неизвестный префикс в Pymophy2.
        // Однако мы можем найти их по постфиксу, потому и сохраняем. + часть из них требует нормализации, которая будет в релизе 0.1.1.
        ParteSpeech::AdjectiveShort, // todo при нормализации сведется к Full
        ParteSpeech::ParticipleShort, // todo при нормализации сведется к Full
        ParteSpeech::Infinitive,
        ParteSpeech::Number,
        ParteSpeech::Adverb,
        ParteSpeech::Comparative, // todo при нормализации сведется к AdjectiveFull
    ];

    /// Предварительный сбор постфиксов (`Vanga`) с наборами тегов.
    ///
    /// У `Vanga`-и может быть не один набор тегов.
    /// Например, `cтали`, окончание `и` - сущ. в род.пад. **и** гл. мн.ч. в прош.вр.
    pub(crate) fn collect_vangas(
        &self,
        id: u64,
        stems: &mut Vec<VangaIntermediate>,
    ) -> Result<(), DictionaryErr> {
        let mut items = Vec::new();
        let stem = longest_common_substring(self.to_longest_common_substring()).to_lowercase();

        // В начальной форме содержатся общие для леммы граммемы, как одушевленность, часть речи и т.п.
        // Эти теги должны быть вычленены и прокинуты всем словам.
        let inizio_grammemes = self.normal_tags()?;

        // Мы можем определять Ванги только у словарных слов.
        if let Some(pos) = Grammem::pos_in_tag(&inizio_grammemes) {
            if Self::VANGA_POS.contains(&pos) {
                match &self
                    .forms
                    .as_ref()
                    .filter(|v| !v.is_empty())
                    .filter(|v| v.iter().all(|v| v.gram.is_some()))
                {
                    None => {
                        // Если у нас нет других форм, кроме начальной, мы сохраняем просто начальную.
                        let normal_form = &self.normal_form.text;
                        let form = Form::Vanga(FVanga::Normal);

                        let vanga_item = VangaItemIntermediate::parse_vanga_item(
                            &stem,
                            normal_form,
                            inizio_grammemes,
                            form,
                        );
                        if let Some(vanga_item) = vanga_item {
                            items.push(vanga_item);
                        }
                    }
                    Some(forms) => {
                        if stem.chars().count() < 3 {
                            // По аналогии с Pymorphy2 мы не собираем Ванги, если оставшаяся основа меньше трех букв.
                        } else {
                            let mut iter = Self::forms(forms.to_owned().to_owned());

                            {
                                let (first, mut first_grammemes) =
                                    iter.next().ok_or(DictionaryErr::NoForms(id))?;
                                first_grammemes.extend(inizio_grammemes.clone());
                                let form = Form::Vanga(FVanga::Normal);

                                let vanga_item = VangaItemIntermediate::parse_vanga_item(
                                    &stem,
                                    &first,
                                    first_grammemes,
                                    form,
                                );
                                if let Some(vanga_item) = vanga_item {
                                    items.push(vanga_item);
                                }
                            }

                            for (text, mut grammemes) in iter {
                                grammemes.extend(inizio_grammemes.clone());
                                let form = Form::Vanga(FVanga::Different);
                                let vanga_item = VangaItemIntermediate::parse_vanga_item(
                                    &stem, &text, grammemes, form,
                                );
                                if let Some(vanga_item) = vanga_item {
                                    items.push(vanga_item);
                                }
                            }
                        }
                    }
                }

                stems.push(VangaIntermediate {
                    popularity: 0,
                    postfix: items,
                })
            }
        } else {
            error!("todo")
        }

        Ok(())
    }
}

/// Известные приставки русского языка, взятые с Википедии.
pub const KNOWN_PREFIX: [&str; 76] = [
    "в",
    "во",
    "взо",
    "вне",
    "внутри",
    "возо",
    "вы",
    "до",
    "еже",
    "за",
    "зако",
    "изо",
    "испод",
    "к",
    "кое",
    "ку",
    "меж",
    "междо",
    "между",
    "на",
    "над",
    "надо",
    "наи",
    "не",
    "недо",
    "ни",
    "низо",
    "о",
    "об",
    "обо",
    "около",
    "от",
    "ото",
    "па",
    "пере",
    "по",
    "под",
    "подо",
    "поза",
    "после",
    "пра",
    "пред",
    "преди",
    "предо",
    "про",
    "противо",
    "разо",
    "с",
    "со",
    "сверх",
    "среди",
    "су",
    "тре",
    "у",
    "без",
    "бес",
    "вз",
    "вс",
    "воз",
    "вос",
    "из",
    "ис",
    "низ",
    "нис",
    "обез",
    "обес",
    "раз",
    "рас",
    "роз",
    "рос",
    "через",
    "черес",
    "чрез",
    "чрес",
    "пре",
    "при",
];

impl Dictionary {
    /// Преобразование по найденному слову и аффиксу
    // fn founded_unafix_word(
    //     &self,
    //     word: &str,
    //     i: u64,
    //     affix: &str,
    //     words_vangas: &mut Vec<VangovanieRes>,
    // ) {
    // let samples = self.word_tags.get(i as usize).unwrap();
    // for Parse { tag, normal_form } in samples {
    //     let vanga_res = VangovanieRes {
    //         vanga_id: None,
    //         affix: affix.to_string(),
    //         vec_tags: tag.to_owned(),
    //         score: 0f32,
    //         method: Vangovanie::KnownPrefix,
    //     };
    //     words_vangas.push(vanga_res)
    // }
    // }

    pub fn vangovanie(&self, word: &str) -> MopsResult<Vec<VangovanieRes>> {
        let _affix = String::new();
        let mut words_vangas = Vec::new();

        // Алгоритм работы со словами с дефисом. release 0.1.2
        // if let Some((_first, _second)) = word.split_once('-') {
        // // todo release 0.1.2
        // #[allow(clippy::single_match)]
        // match first {
        // "по-" => {
        //     match self.to_mmap().get(second) {
        //         None => {}
        //         Some(_) => {} //self.founded_unafix_word(second, i, first, &mut words_vangas),
        //     }
        //     return words_vangas;
        // }
        //     _ => {}
        // }
        // todo
        // #[allow(clippy::single_match)]
        // match second {}
        // "-таки" => {
        //     match self.to_mmap().get(first) {
        //     None => {}
        //     Some(_) => {}, // self.founded_unafix_word(first, i, second, &mut words_vangas),
        // }
        // return words_vangas;}
        // ,
        //     _ => {}
        // }
        // todo!("v0.1.1: сделать вангование с дефисом в приставке, постфиксе и двусоставных словах")
        // ;
        // }

        // Первый этап предсказания Pymorphy2. Сначала ищем возможную приставку.
        // todo Это должны быть известные русские приставки.
        for pref in KNOWN_PREFIX.into_iter() {
            if let Some(word) = word.strip_prefix(pref) {
                // По алгоритму Pymorphy2 основа слова после удаления префикса не должна быть меньше 3 букв.
                if word.chars().count() < 3 {
                    continue;
                }

                match MorphAnalyzer::to_bytes_map(&self.fst)?.get(word) {
                    Some(_i) => {
                        todo!();
                        // Если слово найдено, то наверняка это оно (по концепции Pymorphy2).
                        // Нужно вернуть тег со всем словом
                    }
                    None => continue,
                }
            }
        }

        // Второй этап предсказания Pymorphy2. Если не получилось по известным приставкам, попробуем просто отрезать начало. Но не более 5 букв.
        for i in 1..5 {
            let (_affix, word) = word.split_at(i);
            match MorphAnalyzer::to_bytes_map(&self.fst)?.get(word) {
                Some(_i) => {
                    todo!();
                    // Если слово найдено, то наверняка это оно (по концепции Pymorphy2).
                    // Нужно вернуть тег со всем словом
                }
                None => continue,
            }
        }

        for vanga in self.paradigms.iter() {
            let Vanga {
                popularity: _,
                postfix,
            } = vanga;

            for VangaItem {
                postfix,
                tag: _,
                form: _,
            } in postfix
            {
                // Третий этап по Pymorphy2. Смотрим по окончанию слова.
                if word.ends_with(postfix.as_str()) {
                    todo!();
                    // Если мы нашли какой-то суффикс, еще не значит, что он будет самым вероятным.
                    // Нужно собрать еще варианты.
                } else {
                    continue;
                }
            }
        }

        VangovanieRes::sort(&mut words_vangas);

        Ok(words_vangas.to_vec())
    }
}

#[cfg(test)]
mod test_probability {
    use crate::{
        morph::grammemes::{Grammem, ParteSpeech},
        MorphAnalyzer,
    };

    use super::*;
    use test_case::test_case;

    #[ignore = "too large dictionary"]
    #[test]
    fn test_vangovanie() {
        let dict = crate::analyzer::dictionary::test::make_dict("dict.opcorpora.xml");

        let iter = [
            ("бебекает", ParteSpeech::Verb),
            ("козячный", ParteSpeech::AdjectiveFull),
            ("подпримявшийся", ParteSpeech::ParticipleFull),
            ("кекать", ParteSpeech::Infinitive),
            ("обкекаться", ParteSpeech::Infinitive),
            ("кекуля", ParteSpeech::Noun),
            ("жопиться", ParteSpeech::Infinitive),
            ("кенкетка", ParteSpeech::Noun),
            ("квадрокек", ParteSpeech::Noun),
            ("кекно", ParteSpeech::Adverb),
        ];

        for (word, needed_pos) in iter {
            if dict
                .vangovanie(word)
                .unwrap()
                .iter()
                .find_map(|t| Grammem::pos_in_tag(&t.vec_tags))
                .unwrap()
                != needed_pos
            {
                eprintln!(
                    "{word} - {:?}",
                    dict.vangovanie(word)
                        .unwrap()
                        .iter()
                        .map(|t| (Grammem::pos_in_tag(&t.vec_tags), t.score))
                        .collect_vec()
                );
                panic!()
            };
        }

        std::fs::remove_file(dict.fst).unwrap();
    }

    #[ignore = "release v0.1.2"]
    #[test_case("по-хорошему" => ParteSpeech::Adverb)]
    #[test_case("сказал-таки" => ParteSpeech::Verb)]
    #[test_case("человек-паук" => ParteSpeech::Noun)]
    #[test_case("воздушно-канальный" => ParteSpeech::AdjectiveFull)]
    #[test_case("воздушно-канального" => ParteSpeech::AdjectiveFull)]
    #[test_case("интернет-магазина" => ParteSpeech::Noun)]
    fn eprintln_vangovanie_defis(word: &str) -> ParteSpeech {
        let dict = crate::analyzer::dictionary::test::make_dict("data/test/small_dict.xml");
        dict.vangovanie(word)
            .unwrap()
            .iter()
            .find_map(|t| Grammem::pos_in_tag(&t.vec_tags))
            .unwrap()
    }

    #[ignore = "release v0.1.1"]
    #[test_case("бебекает" => "бебекал")]
    #[test_case("кенкеткy" => "кенкетка")]
    fn test_normal_form_vangovanie(word: &str) -> String {
        let dict = crate::analyzer::dictionary::test::make_dict("data/test/small_dict.xml");
        let normal = MorphAnalyzer::init(dict)
            .unwrap()
            .parse(word)
            .unwrap()
            .0
            .first()
            .unwrap()
            .clone()
            .normal_form;
        normal
    }

    #[test_case("быканул", "ул" => "быкан".to_string())]
    #[test_case("бебекает", "ает" => "бебек".to_string())]
    #[test_case("кенкеткой", "ой" => "кенкетк".to_string())]

    fn test_stemming(word: &str, affix: &str) -> String {
        stemming(word, affix)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VangovanieRes {
    pub(crate) vanga_id: Option<u64>,
    pub(crate) affix: String,
    pub(crate) vec_tags: Tag,
    pub(crate) score: f32,
    pub(crate) method: Vangovanie,
}

impl VangovanieRes {
    pub fn sort(vec: &mut [Self]) {
        vec.sort_by(|v1, v2| v1.score.total_cmp(&v2.score));
    }
}

#[cfg(test)]
mod test {
    use itertools::Itertools;

    use crate::{
        analyzer::{dictionary::test::make_dict, Dictionary},
        morph::grammemes::{FVanga, Form},
        MorphAnalyzer,
    };

    use super::{Vanga, VangaItem};

    #[test]
    fn test_compare() {
        let vanga1 = Vanga {
            popularity: 0,
            postfix: vec![
                VangaItem {
                    postfix: "ый".into(),
                    form: Form::Vanga(FVanga::Normal),
                    tag: vec![],
                },
                VangaItem {
                    postfix: "ee".into(),
                    form: Form::Vanga(FVanga::Different),
                    tag: vec![],
                },
                VangaItem {
                    postfix: "eй".into(),
                    form: Form::Vanga(FVanga::Different),
                    tag: vec![],
                },
            ],
        };

        let vanga2 = Vanga {
            popularity: 1,
            postfix: vec![
                VangaItem {
                    postfix: "ый".into(),
                    form: Form::Vanga(FVanga::Normal),
                    tag: vec![1],
                },
                VangaItem {
                    postfix: "ee".into(),
                    form: Form::Vanga(FVanga::Different),
                    tag: vec![],
                },
                VangaItem {
                    postfix: "eй".into(),
                    form: Form::Vanga(FVanga::Different),
                    tag: vec![],
                },
            ],
        };

        let vanga3 = Vanga {
            popularity: 2,
            postfix: vec![
                VangaItem {
                    postfix: "ый".into(),
                    form: Form::Vanga(FVanga::Normal),
                    tag: vec![],
                },
                VangaItem {
                    postfix: "ого".into(),
                    form: Form::Vanga(FVanga::Different),
                    tag: vec![],
                },
                VangaItem {
                    postfix: "eй".into(),
                    form: Form::Vanga(FVanga::Different),
                    tag: vec![],
                },
            ],
        };

        assert!(vanga1.equal(&vanga2));
        assert!(!vanga1.equal(&vanga3));
        assert!(!vanga2.equal(&vanga3));
    }

    #[test]
    fn test_saturation() {
        let mut vanga1 = Vanga {
            popularity: 0,
            postfix: vec![
                VangaItem {
                    postfix: "ый".into(),
                    form: Form::Vanga(FVanga::Normal),
                    tag: vec![],
                },
                VangaItem {
                    postfix: "ee".into(),
                    form: Form::Vanga(FVanga::Different),
                    tag: vec![],
                },
                VangaItem {
                    postfix: "eй".into(),
                    form: Form::Vanga(FVanga::Different),
                    tag: vec![],
                },
            ],
        };

        let vanga2 = Vanga {
            popularity: 1,
            postfix: vec![
                VangaItem {
                    postfix: "ый".into(),
                    form: Form::Vanga(FVanga::Normal),
                    tag: vec![1],
                },
                VangaItem {
                    postfix: "ee".into(),
                    form: Form::Vanga(FVanga::Different),
                    tag: vec![],
                },
                VangaItem {
                    postfix: "eй".into(),
                    form: Form::Vanga(FVanga::Different),
                    tag: vec![],
                },
            ],
        };

        vanga1.saturation(&vanga2);
        assert_eq!(vanga2, vanga1);
    }

    #[test]
    fn test_lemmas() {
        let dict = make_dict("data/test/small_dict.xml");
        let map = MorphAnalyzer::to_bytes_map(&dict.fst);

        let Dictionary {
            meta: _,
            fst,
            paradigms,
            ..
        } = dict;

        for vanga in paradigms.into_iter().take(50) {
            eprintln!(
                "{:?} - {:?}",
                vanga.popularity,
                vanga.postfix.into_iter().map(|i| (i.postfix)).collect_vec()
            );
        }

        // while let Some(_) = stream.next() {
        // let word = String::from_utf8(item.0.to_vec()).unwrap();
        // let id = item.1;
        // let tags = word_parses.get(id as usize).unwrap();
        // eprintln!("{word} - {id} - {}", Grammem::fmt_vec_tag(tags));
        // }

        drop(map);
        std::fs::remove_file(fst).unwrap();
    }

    #[ignore = "too large dictionary"]
    #[test]
    fn test_real_dict() {
        let dict = make_dict("dict.opcorpora.xml");

        let Dictionary {
            meta: _, paradigms, ..
        } = dict;

        for vanga in paradigms {
            eprintln!(
                "{:?} - {:?}",
                vanga.popularity,
                vanga.postfix.into_iter().map(|i| (i.postfix)).collect_vec()
            );
        }

        std::fs::remove_file(dict.fst).unwrap();
    }
}
