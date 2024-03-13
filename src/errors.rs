use crate::analyzer::{Parse, Tag};
use std::path::PathBuf;
use thiserror::Error;

pub type MopsResult<T, E = MopsErr> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum MopsErr {
    #[error("Couldn't open file {file}: {error}")]
    File {
        file: PathBuf,
        error: std::io::Error,
    },

    #[error("IO err -> {0}")]
    IO(#[from] std::io::Error),

    #[error("XML deserialize err -> {0}")]
    XMLde(#[from] quick_xml::DeError),

    #[error("Serde err -> {0}")]
    Serde(#[from] serde_json::error::Error),

    #[error("Mops dictionary err -> {0}")]
    Dictionary(#[from] DictionaryErr),

    #[error("Fst map err -> {0}")]
    FSTMap(#[from] fst::Error),

    #[error("Parse err -> {0}")]
    Parse(#[from] ParseErr),
}

#[derive(Debug, derive_more::Display)]
/// На какой стадии была потеря леммы.
pub enum Cycle {
    Normal,
    Variant,
    Lost,
}

#[derive(Debug, Error)]
/// Ошибки парсинга словаря `OpenCorpora` в словарь `Mops`-а.
pub enum DictionaryErr {
    #[error("Couldn't create outdir {outdir}: {error}")]
    Outdir {
        outdir: PathBuf,
        error: std::io::Error,
    },

    #[error("Fst err -> {0}")]
    FstBuild(#[from] fst::Error),

    #[error("Lemma {0} in cycle {1} was lost during the parsing")]
    LostLemmaId(u64, Cycle),

    #[error("Binary search not found tag: {0:?}")]
    BinaryTag(Tag),

    #[error("Binary search not found vanga's tag: {0:?}")]
    BinaryTagVanga(Tag),

    #[error("Binary search not found lemma: {0}")]
    BinaryLemma(String),

    #[error("Binary search not found lemmas_row: {0:?}")]
    BinaryRow(Vec<u32>),

    #[error("Binary search not found parses: {0:?}")]
    BinaryParse(Vec<Parse>),

    #[error("Normal form {0} hasn't first grammemes")]
    LostFirstGrammemes(String),

    #[error("Empty LemmaVanga")]
    EmptyVanga,

    #[error("No word form in lemma {0}")]
    NoForms(u64),

    #[error("No word form in lemma {0}")]
    NoFormsVanga(String),

    #[error("Error strip suffix in {0}")]
    Stem(String),
}

#[derive(Debug, derive_more::Display)]
pub enum Bound {
    #[display(fmt = "word_parses")]
    WordParses,
    #[display(fmt = "tags")]
    Tags,
    #[display(fmt = "lemmas")]
    Lemmas,
    #[display(fmt = "alphabet")]
    Alphabet,
    #[display(fmt = "lemmas_row")]
    LemmasRow,
}

#[derive(Debug, Error)]
/// Ошибки парсинга слова
pub enum ParseErr {
    #[error("Index of search {idx} more than {vec} len")]
    OutOfBound { idx: u64, vec: Bound },

    #[error("Word from dictionary {0} lost his normal form")]
    LostNormalForm(String),

    #[error("Word '{0}' parse lost lemma id")]
    LostLemmaId(String),

    #[error("Word '{0}' parse lost lemmas row id")]
    LostLemmasRow(usize),

    #[error("Analyzer lost parse {0:?}")]
    LostParse(Tag),

    #[error("To be continued...")]
    FutureRelease,

    #[error("No parte of speech in {0}")]
    NoPos(String),

    #[error("Declension err -> {0}")]
    Declension(DeclensionErr),

    #[error("Binary search not found tag: {0:?}")]
    BinaryTag(Tag),
}

#[derive(Debug, Error)]
pub enum DeclensionErr {
    #[error("Word is empty")]
    EmptyWord,

    #[error("Binary search not found char {0} in alphabet")]
    BinaryChar(char),

    #[error("Index of search {idx} more than {vec} len")]
    OutOfBound { idx: u64, vec: Bound },
}
