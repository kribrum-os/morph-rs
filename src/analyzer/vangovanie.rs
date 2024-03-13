use smallstr::SmallString;
use tracing::debug;

use super::Tag;
use crate::{
    errors::ParseErr,
    morph::{grammemes::Form, UNPRODUCTIVE},
    MorphAnalyzer, Vangovanie, SMALLLEMMA,
};

/// Приставки, которые не меняют парсинга слово.
///
/// Взято из Pymorphy2.
pub const KNOWN_PREFIX: [&str; 144] = [
    "авиа",
    "авто",
    "аква",
    "анти",
    "анти-",
    "антропо",
    "архи",
    "арт",
    "арт-",
    "астро",
    "аудио",
    "аэро",
    "без",
    "бес",
    "био",
    "вело",
    "взаимо",
    "вне",
    "внутри",
    "видео",
    "вице-",
    "вперед",
    "впереди",
    "гекто",
    "гелио",
    "гео",
    "гетеро",
    "гига",
    "гигро",
    "гипер",
    "гипо",
    "гомо",
    "дву",
    "двух",
    "де",
    "дез",
    "дека",
    "деци",
    "дис",
    "до",
    "евро",
    "за",
    "зоо",
    "интер",
    "инфра",
    "квази",
    "квази-",
    "кило",
    "кино",
    "контр",
    "контр-",
    "космо",
    "космо-",
    "крипто",
    "лейб-",
    "лже",
    "лже-",
    "макро",
    "макси",
    "макси-",
    "мало",
    "меж",
    "медиа",
    "медиа-",
    "мега",
    "мета",
    "мета-",
    "метео",
    "метро",
    "микро",
    "милли",
    "мини",
    "мини-",
    "моно",
    "мото",
    "много",
    "мульти",
    "нано",
    "нарко",
    "не",
    "небез",
    "недо",
    "нейро",
    "нео",
    "низко",
    "обер-",
    "обще",
    "одно",
    "около",
    "орто",
    "палео",
    "пан",
    "пара",
    "пента",
    "пере",
    "пиро",
    "поли",
    "полу",
    "после",
    "пост",
    "пост-",
    "порно",
    "пра",
    "пра-",
    "пред",
    "пресс-",
    "противо",
    "противо-",
    "прото",
    "псевдо",
    "псевдо-",
    "радио",
    "разно",
    "ре",
    "ретро",
    "ретро-",
    "само",
    "санти",
    "сверх",
    "сверх-",
    "спец",
    "суб",
    "супер",
    "супер-",
    "супра",
    "теле",
    "тетра",
    "топ-",
    "транс",
    "транс-",
    "ультра",
    "унтер-",
    "штаб-",
    "экзо",
    "эко",
    "эндо",
    "эконом-",
    "экс",
    "экс-",
    "экстра",
    "экстра-",
    "электро",
    "энерго",
    "этно",
];

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[allow(dead_code)]
pub struct VangovanieRes {
    pub(crate) tags: Tag,
    pub(crate) form: Form,
    pub(crate) normal_form: SmallString<[u8; SMALLLEMMA]>,
    pub(crate) method: Vangovanie,
    pub(crate) score: f32,
}

impl VangovanieRes {
    /// Сортировка результатов Вангования в зависимости от частотности встреченного тега.
    pub fn sort(vec: &mut [Self]) {
        let len = vec.len();

        vec.iter_mut().for_each(|vanga| vanga.score /= len as f32);
        vec.sort_by(|a, b| ((b.score * 100.0) as u8).cmp(&((a.score * 100.0) as u8)));
    }
}

impl MorphAnalyzer {
    /// Предсказание слова, если оно не имеется в словаре.
    ///
    // todo алгоритмы.
    pub fn vangovanie(&self, word: &str) -> Result<Option<Vec<VangovanieRes>>, ParseErr> {
        let mut words_vangas = Vec::new();

        // Алгоритм работы со словами с дефисом. release 0.2.1
        // if let Some((_first, _second)) = word.split_once('-') {
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
        // todo!("сделать вангование с дефисом в приставке, постфиксе и двусоставных словах")
        // ;
        // }

        // Первый этап предсказания Pymorphy2. Сначала ищем возможную приставку.
        for affix in KNOWN_PREFIX.into_iter() {
            if let Some(stem) = word.strip_prefix(affix) {
                // По алгоритму Pymorphy2 основа слова после удаления префикса не должна быть меньше 3 букв.
                if stem.chars().count() < 3 {
                    continue;
                }

                match self.fst.get(stem) {
                    Some(i) => {
                        debug!("Some for {stem}");
                        // Если слово найдено, то наверняка это оно (по концепции Pymorphy2).
                        // Мы собираем эти слова для дальнейшего расчета вероятности.
                        self.founded_unprefix_word(
                            i,
                            Vangovanie::KnownPrefix(affix.into()),
                            &mut words_vangas,
                        )?;
                    }
                    None => {
                        debug!("None for {stem}");
                        continue;
                    }
                }
            }
        }

        // Второй этап предсказания Pymorphy2. Если не получилось по известным приставкам, попробуем просто отрезать начало. Но не более 5 букв.
        for i in 1..5 {
            let affix = word.chars().take(i).collect::<String>();
            if !KNOWN_PREFIX.contains(&affix.as_str()) {
                if let Some(stem) = word.strip_prefix(&affix) {
                    // По алгоритму Pymorphy2 основа слова после удаления префикса не должна быть меньше 3 букв.
                    if stem.chars().count() < 3 {
                        continue;
                    }

                    match self.fst.get(stem) {
                        Some(fst) => {
                            debug!("Some for {stem}");
                            // Если слово найдено, то наверняка это оно (по концепции Pymorphy2).
                            // Мы собираем эти слова для дальнейшего расчета вероятности.
                            self.founded_unprefix_word(
                                fst,
                                Vangovanie::UnknownPrefix(affix.to_owned()),
                                &mut words_vangas,
                            )?;
                        }
                        None => {
                            debug!("None for {stem}");
                            continue;
                        }
                    }
                }
            }
        }

        // todo release 0.2.1.
        // Третий этап предсказания Pymorphy2. Если не получилось по приставкам, попробовать по окончания. Для этого нужны Ванги.
        // TODO: release 0.2.1
        // for (vanga_id, vanga) in self.paradigms.iter().enumerate() {
        //     // По аналогии с Pymorphy2, мы не рассматриваем слишком короткие слова.
        //     if word.chars().count() < 4 {
        //         break;
        //     }
        //     let Vanga {
        //         popularity: _,
        //         postfix,
        //     } = vanga;
        //     for VangaItem { postfix, tag, form } in postfix {
        //         // Третий этап по Pymorphy2. Смотрим по окончанию слова.
        //         if word.strip_suffix(postfix.as_str()).is_some() {
        //             // Если мы нашли какой-то суффикс, еще не значит, что он будет самым вероятным.
        //             // Нужно собрать еще варианты.
        //             // self.founded_unpostfix_word(
        //             //     vanga_id,
        //             //     word,
        //             //     &postfix,
        //             //     tag,
        //             //     form,
        //             //     Vangovanie::Postfix,
        //             //     &mut words_vangas,
        //             //     &mut popularity,
        //             // )?
        //             let mut grammemes = Vec::new();
        //             for tag in tag {
        //                 let tag = self
        //                     .tags
        //                     .get(*tag)
        //                     .ok_or(VangovanieErr::OutOfBound {
        //                         idx: *tag as u64,
        //                         vec: Bound::Tags,
        //                     })?
        //                     .to_owned();
        //                 grammemes.push(tag);
        //             }
        //             if grammemes
        //                 .iter()
        //                 .any(|tags| tags.as_ref().iter().any(|tag| UNPRODUCTIVE.contains(tag)))
        //             {
        //                 error!("Vanga saved unprodictive tag in {tag:?}");
        //             }
        //             for tags in grammemes {
        //                 match form.is_normal() {
        //                     true => {
        //                         let vanga_res = VangovanieRes {
        //                             affix: None,
        //                             tags: tags.clone(),
        //                             form: form.switch_vanga(),
        //                             method: Vangovanie::Postfix,
        //                             normal_form: word.into(),
        //                         };
        //                         let score = popularity.entry(tags).or_insert(0.0);
        //                         *score += 0.5;
        //                         words_vangas.push(vanga_res)
        //                     }
        //                     false => {
        //                         let vanga = self
        //                             .paradigms
        //                             .get(vanga_id)
        //                             .ok_or(VangovanieErr::LostVanga(vanga_id))?;
        //                         let normal = match vanga
        //                             .postfix
        //                             .iter()
        //                             .find(|item| item.form == Form::Vanga(FVanga::Normal))
        //                         {
        //                             Some(item) => item,
        //                             None => {
        //                                 error!("No normal form in {vanga:?}");
        //                                 return Err(VangovanieErr::LostNormalFormVanga(vanga_id));
        //                             }
        //                         };
        //                         let normal_form =
        //                             word.replace(&postfix.to_string(), normal.postfix.as_ref());
        //                         let vanga_res = VangovanieRes {
        //                             affix: None,
        //                             tags: tags.clone(),
        //                             form: form.switch_vanga(),
        //                             method: Vangovanie::Postfix,
        //                             normal_form: normal_form.into(),
        //                         };
        //                         let score = popularity.entry(tags).or_insert(0.0);
        //                         *score += 0.5;
        //                         words_vangas.push(vanga_res)
        //                     }
        //                 }
        //             }
        //         } else {
        //             continue;
        //         }
        //     }
        // }

        if words_vangas.is_empty() {
            Ok(None)
        } else {
            VangovanieRes::sort(&mut words_vangas);
            Ok(Some(words_vangas))
        }
    }

    /// Преобразование по найденному слову и префиксу
    fn founded_unprefix_word(
        &self,
        parse_id: u64,
        method: Vangovanie,
        words_vangas: &mut Vec<VangovanieRes>,
    ) -> Result<(), ParseErr> {
        let samples = self.get_parse(parse_id)?;
        for parse in samples {
            let tags = self.get_tag(parse.tag)?.to_owned();
            if tags.iter().any(|tag| UNPRODUCTIVE.contains(tag)) {
                continue;
            }

            // todo release 0.2.1: корректный способ подсчета
            let score = match method {
                Vangovanie::KnownPrefix(_) => 0.75,
                Vangovanie::UnknownPrefix(_) => 0.5,
                Vangovanie::Postfix => 0.5,
            };

            let vanga_res = VangovanieRes {
                tags: tags.clone(),
                form: parse.form.switch_vanga(),
                method: method.clone(),
                normal_form: self.get_lemmas(parse.normal_form)?.to_owned(),
                score,
            };

            if !words_vangas.contains(&vanga_res) {
                words_vangas.push(vanga_res)
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test_probability {
    use tempfile::tempdir;

    use crate::{
        morph::grammemes::{Grammem, ParteSpeech},
        test_infrastructure::infrastructure::make_dict,
        MorphAnalyzer,
    };
    use itertools::Itertools;
    use test_case::test_case;

    #[ignore = "release 0.2.1"]
    #[test]
    fn test_vangovanie() {
        let anal = MorphAnalyzer::open("data/result/").unwrap();

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
            let t = anal
                .vangovanie(word)
                .expect("No errors in unwrap")
                .expect("Some")
                .into_iter()
                .map(|t| Some((t.clone(), Grammem::pos_in_tag(&t.tags))))
                .next()
                .flatten();
            if let Some((t, pos)) = t.to_owned() {
                if pos.is_none() {
                    eprintln!("Res is {t:?}");
                } else {
                    let pos = pos.unwrap();
                    if pos != needed_pos {
                        eprintln!(
                            "{word} - {:?}",
                            anal.vangovanie(word)
                                .unwrap()
                                .expect("Some")
                                .iter()
                                .map(|t| (Grammem::pos_in_tag(&t.tags)))
                                .collect_vec()
                        );
                        panic!()
                    };
                }
            }
        }
    }

    #[ignore = "release v0.2.1"]
    #[test_case("по-хорошему" => ParteSpeech::Adverb)]
    #[test_case("сказал-таки" => ParteSpeech::Verb)]
    #[test_case("человек-паук" => ParteSpeech::Noun)]
    #[test_case("воздушно-канальный" => ParteSpeech::AdjectiveFull)]
    #[test_case("воздушно-канального" => ParteSpeech::AdjectiveFull)]
    #[test_case("интернет-магазина" => ParteSpeech::Noun)]
    fn eprintln_vangovanie_defis(word: &str) -> ParteSpeech {
        let tmp_dir = tempdir().unwrap();
        let fst = tmp_dir.path().join("defis.fst");

        let dict = make_dict("data/test/small_dict.xml", fst.clone());
        let anal = MorphAnalyzer::init(dict, fst).unwrap();
        anal.vangovanie(word)
            .unwrap()
            .expect("Some")
            .iter()
            .find_map(|t| Grammem::pos_in_tag(&t.tags))
            .unwrap()
    }

    #[ignore = "release v0.2.1"]
    #[test_case("бебекает" => "бебекать")]
    #[test_case("оттеплело" => "оттеплеть")]
    #[test_case("кенкеткy" => "кенкетка")]
    fn test_normal_form_vangovanie_full(word: &str) -> String {
        let normal = MorphAnalyzer::open("data/result/")
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
}

#[cfg(test)]
mod test {
    use itertools::Itertools;

    use crate::MorphAnalyzer;

    #[ignore = "Too large dictionary. Manual test"]
    #[test]
    fn test_real_dict() {
        let anal = MorphAnalyzer::open("data/result/").expect("Mops open");

        for vanga in anal.paradigms {
            eprintln!(
                "{:?} - {:?}",
                vanga.popularity,
                vanga
                    .postfix
                    .into_iter()
                    .map(|i| (
                        i.postfix,
                        i.form,
                        i.tag
                            .into_iter()
                            .map(|tag| anal.tags.get(tag).unwrap())
                            .collect_vec()
                    ))
                    .collect_vec()
            );
        }
    }
}
