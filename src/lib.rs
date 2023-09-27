/// Парсинг слова, предугадывание слова.
pub(crate) mod analyzer;
pub mod errors;
/// Грамматические структуры русского языка, используемые анализатором.
pub(crate) mod morph;
pub use morph::grammemes::*;
/// Словарь Opencorpora.
pub(crate) mod opencorpora;

use allocative::Allocative;
use analyzer::{Lemmas, ParseTable, Tag, Tags};
use errors::{MopsErr, MopsResult};
use fst::Map;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::info;

use crate::{
    analyzer::{Dictionary, NormalizedWords, ParsedWords, Vanga},
    opencorpora::DictionaryOpenCorpora,
};
pub use analyzer::{SMALLLEMMA, SMALLTAG, SMALLVANGA};

#[rustfmt::skip]
#[derive(Debug, Clone, Default, clap::Parser, clap::ValueEnum, Serialize, Deserialize, Allocative)]
/// Имеющиеся словарные языки
pub enum Language {
    #[default]
    Russian,
}

#[derive(Debug, Allocative)]
/// Морфологический анализатор, образованный из словаря.
pub struct MorphAnalyzer {
    #[allocative(skip)]
    pub fst: Map<Vec<u8>>,
    #[allocative(skip)]
    pub word_parses: ParseTable,
    #[allocative(skip)]
    pub tags: Tags,
    #[allocative(skip)]
    pub lemmas: Lemmas,
    pub paradigms: Vec<Vanga>,
}

#[derive(Debug, Clone, Copy, derive_more::Display, PartialEq, Eq, PartialOrd, Ord)]
/// Методы парсинга-нормализации слова: по словарю или вангованию.
pub enum Method {
    Dictionary,
    #[display(fmt = "{}", _0.display())]
    Vangovanie(Vangovanie),
}

#[derive(Debug, Clone, Copy, derive_more::Display, PartialEq, Eq, PartialOrd, Ord)]
/// Имеющиеся типы вангования аналогичны Pymorphy2: KnownPrefix, UnknownPrefix, Postfix.
pub enum Vangovanie {
    KnownPrefix,
    UnknownPrefix,
    Postfix,
}

pub type Normalized = String;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// Распознанное слово.
/// На выход дается само слово, набор из граммем и нормальная форма слова.
pub struct ParsedWord {
    word: String,
    tags: Tag,
    normal_form: Normalized,
    method: Method,
}

impl ParsedWord {
    /// Получение слова.
    pub fn word(&self) -> String {
        self.word.to_string()
    }

    /// Получение тега с граммемами.
    pub fn tag(&self) -> Tag {
        self.tags.to_owned()
    }

    /// Получение нормальной формы слова.
    pub fn normal_form(&self) -> Normalized {
        self.normal_form.to_string()
    }

    /// Метод, котором было вычислено слово.
    pub fn method(&self) -> Method {
        self.method
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// Нормализованное слово.
/// На выход дается нормализованное слово и набор из граммем к нему.
pub struct NormalizedWord {
    normal_word: Normalized,
    tags: Tag,
    method: Method,
}

impl NormalizedWord {
    /// Получение слова.
    pub fn word(&self) -> Normalized {
        self.normal_word.to_string()
    }

    /// Получение тега с граммемами.
    pub fn tag(&self) -> Tag {
        self.tags.to_owned()
    }

    /// Метод, котором было вычислено слово.
    pub fn method(&self) -> Method {
        self.method
    }
}

/// Основная функциональность связана с поиском всех возможных тэгов и нормальных форм.
impl MorphAnalyzer {
    /// Первичное создание словаря.
    ///
    /// `dict_path` - путь до словаря OpenCorpora \
    /// `out_dir` - место, где будет хранится fst и бинарная часть словаря для будущего открытия \
    /// `language` - язык, по-дефолту и пока единственный, Русский.
    pub fn create(dict_path: PathBuf, out_dir: PathBuf, lang: Language) -> MopsResult<Dictionary> {
        let dictionary = DictionaryOpenCorpora::init_from_path(&dict_path)?;
        let dictionary = Dictionary::init(dictionary, &out_dir, lang)?;

        info!("Dictionary was created");
        Ok(dictionary)
    }

    pub fn init(dictionary: Dictionary) -> MopsResult<Self> {
        Self::from_dictionary(dictionary)
    }

    /// Первичное создание словаря по переданному пути с чтением из буфера.
    ///
    /// Чтение из буфера несколько медленнее, чем из строки, но занимает сильно меньше памяти.
    ///
    /// `dict_path` - путь до словаря OpenCorpora \
    /// `out_dir` - место, где будет хранится fst и бинарная часть словаря для будущего открытия \
    /// `language` - язык, по-дефолту и пока единственный, Русский.
    pub fn init_with_reader(
        dict_path: PathBuf,
        out_dir: PathBuf,
        lang: Language,
    ) -> MopsResult<Dictionary> {
        let dictionary = DictionaryOpenCorpora::init_from_path_with_reader(&dict_path)?;
        let dictionary = Dictionary::init(dictionary, &out_dir, lang)?;

        info!("Dictionary was created");
        Ok(dictionary)
    }

    /// Открытие словаря из бинарных данных.
    pub fn open(path: PathBuf) -> MopsResult<Self> {
        let dictionary: Dictionary = Dictionary::open(path)?;
        Self::init(dictionary)
    }

    /// Парсинг слова. Получение всех возможных результатов.
    ///
    /// Все варианты парсинга возвращаются в отсортированном порядке,
    /// гарантируя единообразие выдачи между запусками.
    pub fn parse(&self, word: &str) -> MopsResult<ParsedWords> {
        self.parse_word(word).map_err(MopsErr::Parse)
    }

    /// Нормализация слова. Получение всех возможных результатов.
    ///
    /// Все варианты нормализации возвращаются в отсортированном порядке,
    /// гарантируя единообразие выдачи между запусками.
    pub fn normalize(&self, word: &str) -> MopsResult<NormalizedWords> {
        self.normalized_word(word).map_err(MopsErr::Parse)
    }

    /// v0.1.0 - аналогично `normalize()`
    /// Приведение слова к начальной форме.
    pub fn inflect(&self, word: &str) -> MopsResult<NormalizedWords> {
        self.normalize(word)
    }

    /// Проверка слова на наличие в словаре.
    pub fn is_known(&self, word: &str) -> bool {
        let map = &self.fst;
        map.get(word).is_some()
    }

    /// Парсинг слова и взятие нужного по индексу набора граммем.
    pub fn parse_get(&self, word: &str, index: usize) -> MopsResult<Option<ParsedWord>> {
        Ok(self.parse(word)?.0.get(index).map(|w| w.to_owned()))
    }

    /// Парсинг слова и взятие нужного слова в зависимости от нужного набора граммем.
    pub fn parse_grammemes(
        &self,
        word: &str,
        grammemes: Vec<Grammem>,
    ) -> MopsResult<Option<ParsedWord>> {
        let parsed = self.parse(word)?;

        Ok(parsed.find(grammemes))
    }

    /// Нормализация слова и взятие по индексу формы слова с ее граммемами.
    pub fn normalize_get(&self, word: &str, index: usize) -> MopsResult<Option<NormalizedWord>> {
        Ok(self.normalize(word)?.0.get(index).map(|w| w.to_owned()))
    }

    /// Нормализация слова и взятие нужного слова в зависимости от нужного набора граммем.
    pub fn normalize_grammemes(
        &self,
        word: &str,
        grammemes: Vec<Grammem>,
    ) -> MopsResult<Option<NormalizedWord>> {
        let normalized = self.normalize(word)?;

        Ok(normalized.find(grammemes))
    }
}
