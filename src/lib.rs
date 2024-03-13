/// Парсинг слова, предугадывание слова.
pub(crate) mod analyzer;
pub mod errors;
/// Грамматические структуры русского языка, используемые анализатором.
#[macro_use]
pub mod morph;
/// Словарь Opencorpora.
pub(crate) mod opencorpora;
/// Инфраструктура для юнит-тестов + экспериментальное тестирование.
pub(crate) mod test_infrastructure;

use allocative::Allocative;
use analyzer::{InflectWords, Lemmas, LemmasRows, ParseTable, Tag, Tags};
use errors::{MopsErr, MopsResult};
use fst::Map;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::info;

use crate::{
    analyzer::{Dictionary, Vanga},
    morph::grammemes::Grammem,
    opencorpora::DictionaryOpenCorpora,
};
pub use analyzer::{NormalizedWords, ParsedWords, SMALLLEMMA, SMALLTAG, SMALLVANGA};

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
    pub lemmas_rows: LemmasRows,
}

#[derive(
    Debug, Clone, derive_more::Display, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
/// Методы парсинга-нормализации слова: по словарю или вангованию.
pub enum Method {
    Dictionary,
    #[display(fmt = "{}", _0.display())]
    Vangovanie(Vangovanie),
}

#[derive(
    Debug, Clone, derive_more::Display, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
/// Имеющиеся типы вангования аналогичны Pymorphy2: KnownPrefix, UnknownPrefix, Postfix.
pub enum Vangovanie {
    #[display(fmt = "KnowPrefix({_0})")]
    KnownPrefix(String),
    #[display(fmt = "UnknowPrefix({_0})")]
    UnknownPrefix(String),
    Postfix,
}

pub type Normalized = String;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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
        self.word.to_owned()
    }

    /// Получение тега с граммемами.
    pub fn tag(&self) -> Tag {
        self.tags.to_owned()
    }

    /// Получение нормальной формы слова.
    pub fn normal_form(&self) -> Normalized {
        self.normal_form.to_owned()
    }

    /// Метод, которым было вычислено слово.
    pub fn method(&self) -> Method {
        self.method.to_owned()
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
        self.normal_word.to_owned()
    }

    /// Получение тега с граммемами.
    pub fn tag(&self) -> Tag {
        self.tags.to_owned()
    }

    /// Метод, которым было вычислено слово.
    pub fn method(&self) -> Method {
        self.method.to_owned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Измененная форма слова.
/// Может быть просто начальной формой слова,
/// может быть словом в определенном склонении.
pub struct InflectWord {
    inflect_form: String,
    tags: Tag,
    normal_form: Normalized,
    method: Method,
}

impl InflectWord {
    /// Получение слова.
    pub fn word(&self) -> String {
        self.inflect_form.to_owned()
    }

    /// Получение тега с граммемами.
    pub fn tag(&self) -> Tag {
        self.tags.to_owned()
    }

    pub fn method(&self) -> Method {
        self.method.to_owned()
    }
}

/// Основная функциональность связана с разбиением слова по морфемам,
/// определением парадигм и поиском всех возможных тэгов и нормальных форм.
impl MorphAnalyzer {
    /// Первичное создание словаря.
    ///
    /// `dict_path` - путь до словаря OpenCorpora \
    /// `out_dir` - место, где будет храниться fst и бинарная часть словаря для будущего открытия \
    /// `language` - язык, по дефолту и пока единственный, Русский.
    pub fn create<P: AsRef<Path>>(
        dict_path: P,
        out_dir: P,
        lang: Language,
    ) -> MopsResult<Dictionary> {
        let dictionary = DictionaryOpenCorpora::init_from_path(dict_path)?;
        let dictionary = Dictionary::init(dictionary, &out_dir, lang)?;

        info!("Dictionary was created");
        Ok(dictionary)
    }

    /// Первичное создание словаря по переданному пути с чтением из буфера.
    ///
    /// Чтение из буфера несколько медленнее, чем из строки, но занимает сильно меньше памяти.
    ///
    /// `dict_path` - путь до словаря OpenCorpora \
    /// `out_dir` - место, где будет храниться fst и бинарная часть словаря для будущего открытия \
    /// `language` - язык, по дефолту и пока единственный, Русский.
    pub fn create_with_reader<P: AsRef<Path>>(
        dict_path: P,
        out_dir: P,
        lang: Language,
    ) -> MopsResult<Dictionary> {
        let dictionary = DictionaryOpenCorpora::init_from_path_with_reader(dict_path)?;
        let dictionary = Dictionary::init(dictionary, out_dir, lang)?;

        info!("Dictionary was created");
        Ok(dictionary)
    }

    /// Инициализация из словаря.
    ///
    /// Отличается от словаря отсутствием мета-информации и
    /// тем, что поднимает в оперативную память fst-словарь.
    pub fn init<P: AsRef<Path>>(dictionary: Dictionary, dir: P) -> MopsResult<Self> {
        let fst = dir.as_ref().join("dict.fst");
        Self::from_dictionary(dictionary, fst)
    }

    /// Открытие словаря из `dict.json` файла и инициализация `MorphAnalyzer`-а.
    pub fn open<P: AsRef<Path>>(path: P) -> MopsResult<Self> {
        let dictionary: Dictionary = Dictionary::open(&path)?;
        Self::init(dictionary, path)
    }

    /// Открытие словаря из `dict.json` файла, используя Reader для файла,
    /// и инициализация `MorphAnalyzer`-а.
    ///
    /// WARN! ОЧЕНЬ долгий процесс чтения.
    /// Рекомендуется использование только при очень ограниченной оперативной памяти.
    pub fn open_from_reader<P: AsRef<Path>>(path: P) -> MopsResult<Self> {
        let dictionary: Dictionary = Dictionary::open_from_reader(&path)?;
        Self::init(dictionary, path)
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

    /// Приведение к начальной форме слова.
    /// Начальная форма может отличаться от нормализованной.
    pub fn inflect_inizio(&self, word: &str) -> MopsResult<Option<InflectWords>> {
        self.inflect_word(word, None).map_err(MopsErr::Parse)
    }

    /// Приведение слова к нужной форме слова с указанными граммемами.
    pub fn inflect_forms(
        &self,
        word: &str,
        grammemes: Vec<Grammem>,
    ) -> MopsResult<Option<InflectWords>> {
        self.inflect_word(word, Some(grammemes))
            .map_err(MopsErr::Parse)
    }

    /// Приведение разобранного слова к нужной форме слова с указанными граммемами.
    pub fn inflect_parsed(
        &self,
        parse: ParsedWord,
        grammemes: Vec<Grammem>,
    ) -> MopsResult<Option<InflectWords>> {
        self.inflect_parsed_words(parse, Some(grammemes))
            .map_err(MopsErr::Parse)
    }

    /// Полное склонение/спряжение слова по всем формам.
    ///
    /// WARN: Не быстрая функция. Если есть необходимый набор слов,
    /// который нужно будет искать во всех формах в тексте, лучше сделать вызов `declension()`
    /// ко всем словам в начале работы приложения.
    pub fn declension(&self, word: &str) -> MopsResult<Vec<InflectWords>> {
        self.declension_word(word).map_err(MopsErr::Parse)
    }

    /// Полное склонение/спряжение слова и взятие нужного по индексу результата.
    ///
    /// WARN: Не быстрая функция. Если есть необходимый набор слов,
    /// который нужно будет искать во всех формах в тексте, лучше сделать вызов `declension_get()`
    /// ко всем словам в начале работы приложения.
    pub fn declension_get(&self, word: &str, index: usize) -> MopsResult<Option<InflectWords>> {
        self.declension(word)
            .map(|w| w.get(index).map(|p| p.to_owned()))
    }

    /// Полное склонение/спряжение разобранного слова.
    ///
    /// WARN: Не быстрая функция. Если есть необходимый набор слов,
    /// который нужно будет искать во всех формах в тексте, лучше сделать вызов `declension_parsed()`
    /// ко всем словам в начале работы приложения.
    pub fn declension_parsed(&self, parse: &ParsedWord) -> MopsResult<Option<InflectWords>> {
        self.declension_parsed_word(parse).map_err(MopsErr::Parse)
    }
}
