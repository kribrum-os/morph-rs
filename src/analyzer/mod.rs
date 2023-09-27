use crate::{
    errors::{Bound, MopsErr, MopsResult, ParseErr},
    morph::grammemes::{FWord, Form, Grammem},
    Method, MorphAnalyzer, NormalizedWord, ParsedWord,
};
use allocative::Allocative;
use fst::Map;
use serde::{Deserialize, Serialize};
use smallstr::SmallString;
use smallvec::SmallVec;
use std::path::PathBuf;
use tracing::debug;

// Значения для Small-хранения постфиксов ванги, граммемов в теге, лемм.
// Нынешние значения вычислены экспериментально и могут меняться при дальнейших экспериментах.

/// Количество байт, которое вмещает в себя большую часть постфиксов Ванги,
/// чтобы не аллоцировать под небольшой размер данных большое количество места на куче.
pub const SMALLVANGA: usize = 8;
/// Количество байт, которое вмещает в себя большую часть тегов
/// чтобы не аллоцировать под небольшой размер данных большое количество места на куче.
pub const SMALLTAG: usize = 8;
/// Количество байт, которое вмещает в себя большую часть лемм (нормальных форм слова)
/// чтобы не аллоцировать под небольшой размер данных большое количество места на куче.
pub const SMALLLEMMA: usize = 16;

/// Сборка словаря
mod dictionary;
pub mod pretty_display;
pub use dictionary::Dictionary;
/// Vanga и всё с ней связанное. // todo v0.2.0
pub(crate) mod vangovanie;

/// Набор граммем слова.
pub type Tag = SmallVec<[Grammem; SMALLTAG]>;
/// Все наборы тегов
pub type Tags = Vec<Tag>;
/// Все нормализованные слова.
pub type Lemmas = Vec<SmallString<[u8; SMALLLEMMA]>>;

/// Структура хранения всех разборов слов.
/// TagsId в fst ссылается на нее.
pub type ParseTable = Vec<Vec<Parse>>;

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Clone, Serialize, Deserialize, Allocative)]
/// Один разбор слова: форма, набор тегов, нормализованная форма.
pub struct Parse {
    pub(crate) form: Form,
    pub(crate) tag: TagID,
    pub(crate) normal_form: LemmaID,
}

/// Index в Tags
pub type TagID = usize;
/// Index в Lemmas
pub type LemmaID = usize;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Serialize, Deserialize, Allocative)]
/// `Vanga` - предсказание по части речи на основе постфикса.
pub struct Vanga {
    pub(crate) popularity: u64,
    pub postfix: Vec<VangaItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Serialize, Deserialize, Allocative)]
pub struct VangaItem {
    #[allocative(skip)]
    pub postfix: SmallString<[u8; SMALLVANGA]>,
    pub(crate) form: Form,
    pub(crate) tag: Vec<TagID>,
}

impl MorphAnalyzer {
    /// Создание анализатора из словаря.
    pub fn from_dictionary(dictionary: Dictionary) -> MopsResult<Self> {
        let Dictionary {
            meta: _,
            fst,
            word_parses,
            tags,
            lemmas,
            paradigms,
        } = dictionary;

        Ok(Self {
            fst: Self::to_bytes_map(&fst)?,
            word_parses,
            tags,
            lemmas,
            paradigms,
        })
    }

    /// Взятие бинарного представления из словаря на диске в RAM.
    pub(crate) fn to_bytes_map(fst: &PathBuf) -> MopsResult<Map<Vec<u8>>> {
        let buf = std::fs::read(fst).map_err(MopsErr::IO)?;
        Map::new(buf).map_err(MopsErr::FSTMap)
    }

    /// Парсинг слова.
    pub fn parse_word(&self, word: &str) -> Result<ParsedWords, ParseErr> {
        let map = &self.fst;
        let mut parsed = ParsedWords::default();

        match map.get(word.as_bytes()) {
            Some(common_id) => {
                debug!("{word} найдено в словаре");
                let vec_tags =
                    self.word_parses
                        .get(common_id as usize)
                        .ok_or(ParseErr::OutOfBound {
                            idx: common_id,
                            vec: Bound::WordParses,
                        })?;

                for Parse {
                    tag, normal_form, ..
                } in vec_tags
                {
                    parsed.0.push(ParsedWord {
                        word: word.to_string(),
                        tags: self
                            .tags
                            .get(*tag)
                            .ok_or(ParseErr::OutOfBound {
                                idx: *tag as u64,
                                vec: Bound::Tags,
                            })?
                            .to_owned(),
                        normal_form: self
                            .lemmas
                            .get(*normal_form)
                            .ok_or(ParseErr::OutOfBound {
                                idx: *normal_form as u64,
                                vec: Bound::Lemmas,
                            })?
                            .to_string(),
                        method: Method::Dictionary,
                    })
                }

                parsed.0.sort();
                Ok(parsed)
            }
            None => Err(ParseErr::FutureRelease),
        }
    }

    /// Нормализация слова.
    pub fn normalized_word(&self, word: &str) -> Result<NormalizedWords, ParseErr> {
        let map = &self.fst;
        let mut normalized = NormalizedWords::default();

        match map.get(word) {
            Some(common_id) => {
                debug!("{word} найдено в словаре");
                let vec_parses =
                    self.word_parses
                        .get(common_id as usize)
                        .ok_or(ParseErr::OutOfBound {
                            idx: common_id,
                            vec: Bound::WordParses,
                        })?;

                for Parse {
                    tag,
                    normal_form,
                    form,
                } in vec_parses.iter()
                {
                    if matches!(form, Form::Word(FWord::Normal(_))) {
                        let normalized_word = NormalizedWord {
                            normal_word: self
                                .lemmas
                                .get(*normal_form)
                                .ok_or(ParseErr::OutOfBound {
                                    idx: *normal_form as u64,
                                    vec: Bound::Lemmas,
                                })?
                                .to_string(),
                            tags: self
                                .tags
                                .get(*tag)
                                .ok_or(ParseErr::OutOfBound {
                                    idx: *tag as u64,
                                    vec: Bound::Tags,
                                })?
                                .clone(),
                            method: Method::Dictionary,
                        };
                        if !normalized.0.contains(&normalized_word) {
                            normalized.0.push(normalized_word)
                        }
                    } else {
                        let word = self
                            .lemmas
                            .get(*normal_form)
                            .ok_or(ParseErr::OutOfBound {
                                idx: *normal_form as u64,
                                vec: Bound::Lemmas,
                            })?
                            .to_string();
                        let id = map.get(&word).ok_or(ParseErr::LostNormal(word.clone()))?;
                        let vec_parses =
                            self.word_parses
                                .get(id as usize)
                                .ok_or(ParseErr::OutOfBound {
                                    idx: common_id,
                                    vec: Bound::WordParses,
                                })?;

                        for Parse { tag, form, .. } in vec_parses.iter() {
                            if matches!(form, Form::Word(FWord::Normal(_))) {
                                let normalized_word = NormalizedWord {
                                    normal_word: word.clone(),
                                    tags: self
                                        .tags
                                        .get(*tag)
                                        .ok_or(ParseErr::OutOfBound {
                                            idx: *tag as u64,
                                            vec: Bound::Tags,
                                        })?
                                        .to_owned(),
                                    method: Method::Dictionary,
                                };
                                // Исключаем повторы для, например, существительных
                                if !normalized.0.contains(&normalized_word) {
                                    normalized.0.push(normalized_word)
                                }
                            }
                        }
                    }
                }

                normalized.0.sort();
                Ok(normalized)
            }
            None => Err(ParseErr::FutureRelease),
        }
    }
}

#[derive(Default)]
/// Вектор распознанных слов.
pub struct ParsedWords(pub Vec<ParsedWord>);

impl ParsedWords {
    pub fn find(self, memes: Vec<Grammem>) -> Option<ParsedWord> {
        self.0
            .into_iter()
            .find(|w| memes.iter().all(|meme| w.clone().tag().contains(meme)))
    }
}

#[derive(Default)]
/// Вектор нормализованных слов.
pub struct NormalizedWords(pub Vec<NormalizedWord>);

impl NormalizedWords {
    pub fn find(self, memes: Vec<Grammem>) -> Option<NormalizedWord> {
        self.0
            .into_iter()
            .find(|w| memes.iter().all(|meme| w.clone().tag().contains(meme)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        morph::grammemes::{Gender, ParteSpeech},
        Method,
    };

    #[test]
    fn test_find_parsed() {
        let parsed1 = ParsedWord {
            word: "bebeka".to_string(),
            tags: SmallVec::from(vec![
                Grammem::ParteSpeech(ParteSpeech::Noun),
                Grammem::Gender(Gender::Feminine),
            ]),
            normal_form: "bebe".to_string(),
            method: Method::Vangovanie(crate::Vangovanie::Postfix),
        };

        let parsed2 = ParsedWord {
            word: "bebek".to_string(),
            tags: SmallVec::from(vec![
                Grammem::ParteSpeech(ParteSpeech::Noun),
                Grammem::Gender(Gender::Masculine),
            ]),
            normal_form: "bebe".to_string(),
            method: Method::Vangovanie(crate::Vangovanie::Postfix),
        };

        let parsed3 = ParsedWord {
            word: "bebeki".to_string(),
            tags: SmallVec::from(vec![Grammem::ParteSpeech(ParteSpeech::Noun)]),
            normal_form: "bebe".to_string(),
            method: Method::Vangovanie(crate::Vangovanie::Postfix),
        };

        let words = ParsedWords(vec![parsed1.clone(), parsed2, parsed3]);
        assert_eq!(
            parsed1,
            words
                .find(vec![
                    Grammem::ParteSpeech(ParteSpeech::Noun),
                    Grammem::Gender(Gender::Feminine)
                ])
                .unwrap()
        )
    }
}
