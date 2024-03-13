use crate::{
    analyzer::vangovanie::VangovanieRes,
    errors::{MopsErr, MopsResult, ParseErr},
    morph::grammemes::{Form, Grammem},
    InflectWord, Method, MorphAnalyzer, NormalizedWord, ParsedWord, Vangovanie,
};
use allocative::Allocative;
use fst::Map;
use serde::{Deserialize, Serialize};
use smallstr::SmallString;
use smallvec::SmallVec;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};
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
pub(crate) mod dictionary;
pub use dictionary::Dictionary;

/// Сборка префиксного поиска по fst::Stream для улучшения производительности.
pub(crate) mod declension;
/// Вспомогательные функции морфологизатора для разборов слов.
pub(crate) mod morpholyzer;
/// Предугадывание слов.
pub(crate) mod vangovanie;

pub mod pretty_display;

/// Набор граммем слова.
pub type Tag = SmallVec<[Grammem; SMALLTAG]>;
/// Все наборы тегов
pub type Tags = Vec<Tag>;
/// Все нормализованные слова.
pub type Lemmas = Vec<SmallString<[u8; SMALLLEMMA]>>;

/// Структура хранения всех разборов слов.
/// Id в fst ссылается на нее.
pub type ParseTable = Vec<Vec<Parse>>;

/// OpenCorpora's LemmaId.
pub type OpCLid = u32;
/// Все слитые между собой для нормализации OpenCorpora's LemmaId.
pub type LemmasRows = Vec<Vec<OpCLid>>;

#[derive(
    Debug, PartialEq, Eq, Ord, PartialOrd, Clone, Serialize, Deserialize, Allocative, Hash,
)]
/// Один разбор слова: форма, набор тегов, нормализованная форма.
pub struct Parse {
    pub(crate) form: Form,
    pub(crate) tag: TagID,
    pub(crate) normal_form: LemmaID,
    pub(crate) lemma_row_id: LemmaRowId,
}

/// Index в Tags
pub type TagID = usize;
/// Index в Lemmas
pub type LemmaID = usize;
/// Index в LemmasRows.
pub type LemmaRowId = usize;

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

#[derive(Debug, Default, Eq, PartialEq, Clone)]
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

#[derive(Debug, Default, Clone)]
/// Вектор слов в соответствующей форме.
pub struct InflectWords(pub Vec<InflectWord>);

impl InflectWords {
    pub fn find(self, memes: Vec<Grammem>) -> Option<InflectWord> {
        self.0
            .into_iter()
            .find(|w| memes.iter().all(|meme| w.clone().tag().contains(meme)))
    }
}

impl MorphAnalyzer {
    /// Создание анализатора из словаря.
    pub fn from_dictionary(dictionary: Dictionary, fst: PathBuf) -> MopsResult<Self> {
        let Dictionary {
            meta: _,
            word_parses,
            tags,
            lemmas,
            paradigms,
            lemmas_rows,
        } = dictionary;

        Ok(Self {
            fst: Self::to_bytes_map(&fst)?,
            word_parses,
            tags,
            lemmas,
            paradigms,
            lemmas_rows,
        })
    }

    /// Взятие бинарного представления из словаря на диске в RAM.
    pub(crate) fn to_bytes_map(fst: &PathBuf) -> MopsResult<Map<Vec<u8>>> {
        let buf = std::fs::read(fst).map_err(|error| MopsErr::File {
            file: fst.to_path_buf(),
            error,
        })?;
        Map::new(buf).map_err(MopsErr::FSTMap)
    }

    /// Парсинг слова.
    pub fn parse_word(&self, word: &str) -> Result<ParsedWords, ParseErr> {
        let map = &self.fst;
        let mut parsed = ParsedWords::default();

        match map.get(word.as_bytes()) {
            Some(common_id) => {
                debug!("{word} найдено в словаре");
                let vec_tags = self.get_parse(common_id)?;

                for parse in vec_tags {
                    parsed.0.push(self.try_into_parse(word, parse)?)
                }

                // Для Ванги не должно быть сортировки, т.к. она выводится по score.
                parsed.0.sort();
            }
            None => {
                if let Some(vanga) = self.vangovanie(word)? {
                    for VangovanieRes {
                        tags,
                        form: _,
                        method,
                        normal_form,
                        ..
                    } in vanga
                    {
                        let normal_form = match &method {
                            Vangovanie::KnownPrefix(affix) | Vangovanie::UnknownPrefix(affix) => {
                                format!("{affix}{normal_form}")
                            }
                            Vangovanie::Postfix => return Err(ParseErr::FutureRelease),
                        };

                        parsed.0.push(ParsedWord {
                            word: word.to_string(),
                            tags,
                            normal_form,
                            method: Method::Vangovanie(method),
                        })
                    }
                }
            }
        }

        Ok(parsed)
    }

    /// Нормализация слова.
    pub fn normalized_word(&self, word: &str) -> Result<NormalizedWords, ParseErr> {
        let map = &self.fst;
        let mut normalized = NormalizedWords::default();

        match map.get(word) {
            Some(common_id) => {
                debug!("{word} найдено в словаре");
                let vec_parses = self.get_parse(common_id)?;

                for parse in vec_parses.iter() {
                    if parse.form.is_normal() {
                        normalized.0.push(self.try_into_normalized(parse)?)
                    } else {
                        // Нам нужно брать только те нормальные формы, которые имеют отношение к соответствующему парсингу.
                        let lemmas_link = self.get_row_id(parse.lemma_row_id)?;

                        let word = self.get_lemmas(parse.normal_form)?.to_string();
                        let id = map
                            .get(&word)
                            .ok_or_else(|| ParseErr::LostNormalForm(word.clone()))?;
                        let vec_parses = self.get_parse(id)?;

                        for parse in vec_parses.iter() {
                            let normalized_word = self.try_into_normalized(parse)?;
                            if parse.form.is_normal()
                                && !normalized.0.contains(&normalized_word)
                                // Нам нужно брать только те нормальные формы, которые имеют отношение к соответствующему парсингу.
                                && lemmas_link.contains(&(parse.form.id().unwrap() as u32))
                            {
                                normalized.0.push(normalized_word)
                            }
                        }
                    }
                }

                // Для Ванги не должно быть сортировки, т.к. она выводится по score.
                normalized.0.sort();
            }
            None => {
                if let Some(vanga) = self.vangovanie(word)? {
                    for VangovanieRes {
                        tags, form, method, ..
                    } in vanga
                    {
                        if form.is_normal() {
                            normalized.0.push(NormalizedWord {
                                normal_word: word.to_owned(),
                                tags,
                                method: Method::Vangovanie(method),
                            })
                        } else {
                            return Err(ParseErr::FutureRelease);
                        }
                    }
                }
            }
        }

        Ok(normalized)
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct WordForm<'a> {
    i: u64,
    tag: &'a Tag,
    lemma: &'a SmallString<[u8; SMALLLEMMA]>,
}

impl MorphAnalyzer {
    /// Приведение слова к нужной форме, указанной через граммемы.
    ///
    /// Если граммемы не указаны, слово будет приведено к начальной форме (чаще всего, им.п., ед.ч. и т.п.).
    /// Начальная форма не является нормализацией слова.
    pub(crate) fn inflect_word(
        &self,
        word: &str,
        grammemes: Option<Vec<Grammem>>,
    ) -> Result<Option<InflectWords>, ParseErr> {
        let map = &self.fst;
        let mut inflect = InflectWords::default();

        match map.get(word) {
            Some(common_id) => {
                debug!("{word} найдено в словаре");
                let vec_parses = self.get_parse(common_id)?;

                // Для каждого парсинга слова нам нужен свой набор элементов.
                for parse in vec_parses.iter() {
                    self.inflect_parse(word, parse, grammemes.clone(), &mut inflect)?;
                }
            }
            None => return Err(ParseErr::FutureRelease),
        };

        if inflect.0.is_empty() {
            Ok(None)
        } else {
            Ok(Some(inflect))
        }
    }

    /// Привести разобранное слово к нужной форме, указанной через граммемы.
    ///
    /// Если граммемы не указаны, слово будет приведено к начальной форме (чаще всего, им.п., ед.ч. и т.п.).
    /// Начальная форма не является нормализацией слова.
    pub(crate) fn inflect_parsed_words(
        &self,
        word: ParsedWord,
        grammemes: Option<Vec<Grammem>>,
    ) -> Result<Option<InflectWords>, ParseErr> {
        let map = &self.fst;
        let mut inflect = InflectWords::default();

        match map.get(word.word()) {
            Some(common_id) => {
                let tag = self
                    .tags
                    .binary_search(&word.tag())
                    .map_err(|_| ParseErr::BinaryTag(word.tag()))?;
                let parse = self
                    .get_parse(common_id)?
                    .iter()
                    .find(|parse| parse.tag == tag)
                    .ok_or_else(|| ParseErr::LostParse(word.tag()))?;

                self.inflect_parse(&word.word(), parse, grammemes, &mut inflect)?;
            }
            None => return Err(ParseErr::FutureRelease),
        }

        if inflect.0.is_empty() {
            Ok(None)
        } else {
            Ok(Some(inflect))
        }
    }

    /// По имеющемуся разбору слова и грамматическим ограничениям (`Vec<Grammem>`)
    /// собирается измененная форма слова.
    ///
    /// Если грамматических ограничений нет, возвращается начальная форма слова.
    fn inflect_parse(
        &self,
        word: &str,
        parse: &Parse,
        grammemes: Option<Vec<Grammem>>,
        inflect: &mut InflectWords,
    ) -> Result<(), ParseErr> {
        // Если граммемы не переданы, требуется начальная форма. Она, в свою очередь, может совпадать с нормальной.
        if grammemes.is_none() && (parse.form.is_inizio() || parse.form.is_normal()) {
            inflect
                .0
                .push(self.try_into_inflect(word.to_string(), parse)?);
        } else {
            // Нам нужно брать только те формы, которые имеют отношение к соответствующему парсингу.
            let ids = self.get_row_id(parse.lemma_row_id)?.to_owned();
            // Если мы ищем начальную форму, нам понадобится не выходить за пределы Opencorpora's Lemma Id слова.
            let word_id = parse
                .form
                .id()
                .ok_or_else(|| ParseErr::LostLemmaId(word.to_string()))?;

            let mut hash_set: HashMap<(String, Option<String>), Vec<WordForm>> = HashMap::new();

            let id_forms = self.id_forms(word, &ids, Some(word_id), &grammemes);
            self.collect_stream_hashset(word, &grammemes, id_forms, &mut hash_set)?;
            self.iter_fst(&mut hash_set, inflect)?;
        }

        Ok(())
    }

    /// Проход по всем склонениям/спряжениям слова.
    ///
    /// Для каждого разбора данного слова возвращается набор `ParsedWords` всех склонений-спряжений, связанных с каждым его разбором.
    ///
    /// # Example
    /// стали (металл) -> сталь, стали, стали, сталь, сталью, стали \
    /// стали (как стать) -> стать, стал, стала, стали.
    ///
    /// ### Warn!
    /// Не быстрая функция.
    pub(crate) fn declension_word(&self, word: &str) -> Result<Vec<InflectWords>, ParseErr> {
        let map = &self.fst;
        let mut inflects = Vec::new();

        match map.get(word.as_bytes()) {
            Some(common_id) => {
                let set_ids = self
                    .get_parse(common_id)?
                    .iter()
                    .filter_map(|parse| self.lemmas_rows.get(parse.lemma_row_id))
                    .map(|v| v.to_owned())
                    .collect::<HashSet<Vec<OpCLid>>>();

                // Нам нужно брать только те формы, которые имеют отношение к соответствующему парсингу.
                for ids in set_ids {
                    let mut inflect = InflectWords::default();
                    self.declension_ids(word, &ids, &mut inflect)?;
                    if !inflect.0.is_empty() {
                        inflects.push(inflect);
                    }
                }
            }

            None => return Err(ParseErr::FutureRelease),
        }

        Ok(inflects)
    }

    /// Проход по всем склонениям/спряжениям разобранного слова.
    ///
    /// # Example
    /// `ParsedWord = 'стали', tags: [Verb, Perfetto, Indicativo, Plural, Intransitive, Past], normal_form = 'стать', Method::Dictionary`
    ///  -> `стать, стал, стала, стали и т.д.`.
    ///
    /// ### Warn!
    /// Не быстрая функция.
    pub(crate) fn declension_parsed_word(
        &self,
        word: &ParsedWord,
    ) -> Result<Option<InflectWords>, ParseErr> {
        let map = &self.fst;
        let mut inflect = InflectWords::default();

        match map.get(word.word()) {
            Some(common_id) => {
                let tag = self
                    .tags
                    .binary_search(&word.tag())
                    .map_err(|_| ParseErr::BinaryTag(word.tag()))?;
                let parse = self
                    .get_parse(common_id)?
                    .iter()
                    .find(|parse| parse.tag == tag)
                    .ok_or_else(|| ParseErr::LostParse(word.tag()))?;

                // Нам нужно брать только те формы, которые имеют отношение к соответствующему парсингу.
                let ids = self.get_row_id(parse.lemma_row_id)?;
                self.declension_ids(&word.word(), ids, &mut inflect)?;
            }
            None => return Err(ParseErr::FutureRelease),
        }

        if inflect.0.is_empty() {
            Ok(None)
        } else {
            Ok(Some(inflect))
        }
    }

    /// Склонение/спряжение всех слов, стоящих в одной связи
    /// (`ids` - id лемм из `OpenCorpora`, которые как-то связаны через `links`).
    ///
    /// При этом используется префиксное ограничение для `fst::Stream`, чтобы сократить время хождения по словарю.
    fn declension_ids(
        &self,
        word: &str,
        ids: &[u32],
        inflect: &mut InflectWords,
    ) -> Result<(), ParseErr> {
        let mut hash_set: HashMap<(String, Option<String>), Vec<WordForm>> = HashMap::new();

        let id_forms = self.id_forms(word, ids, None, &None);
        self.collect_stream_hashset(word, &None, id_forms, &mut hash_set)?;
        self.iter_fst(&mut hash_set, inflect)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        grams,
        morph::grammemes::{Case, Gender, ParteSpeech},
        Method,
    };

    #[test]
    fn test_find_parsed() {
        let parsed1 = ParsedWord {
            word: "bebeka".to_string(),
            tags: SmallVec::from(grams![ParteSpeech::Noun, Gender::Feminine]),
            normal_form: "bebe".to_string(),
            method: Method::Vangovanie(crate::Vangovanie::Postfix),
        };

        let parsed2 = ParsedWord {
            word: "bebek".to_string(),
            tags: SmallVec::from(grams![ParteSpeech::Noun, Gender::Masculine]),
            normal_form: "bebe".to_string(),
            method: Method::Vangovanie(crate::Vangovanie::Postfix),
        };

        let parsed3 = ParsedWord {
            word: "bebeki".to_string(),
            tags: SmallVec::from(grams![ParteSpeech::Noun]),
            normal_form: "bebe".to_string(),
            method: Method::Vangovanie(crate::Vangovanie::Postfix),
        };

        let words = ParsedWords(vec![parsed1.clone(), parsed2, parsed3]);
        assert_eq!(
            parsed1,
            words
                .find(grams![ParteSpeech::Noun, Gender::Feminine])
                .unwrap()
        )
    }

    #[test]
    fn test_inflect_form() {
        let anal = MorphAnalyzer::open("data/result/").unwrap();

        let femn_invest = anal
            .inflect_forms(
                "инвестировавшие",
                grams![Gender::Feminine, Case::Nominativus],
            )
            .unwrap()
            .unwrap();

        assert_eq!(
            "инвестировавшая",
            femn_invest.0.first().unwrap().to_owned().word().as_str()
        );
    }

    #[test]
    fn test_inflect_form_full() {
        let anal = MorphAnalyzer::open("data/result/").unwrap();

        let femn_invest = anal.inflect_inizio("инвестировавшие").unwrap().unwrap();

        assert_eq!(
            "инвестировавший",
            femn_invest.0.first().unwrap().to_owned().word().as_str()
        );
    }
}
