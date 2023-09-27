use thiserror::Error;

use crate::analyzer::{Parse, Tag};

pub type MopsResult<T, E = MopsErr> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum MopsErr {
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

#[derive(Debug, Error)]
pub enum DictionaryErr {
    #[error("IO err -> {0}")]
    IO(#[from] std::io::Error),

    #[error("Fst err -> {0}")]
    FstBuild(#[from] fst::Error),

    #[error("SmallString conversion err -> {0}")]
    SmallString(#[source] core::convert::Infallible),

    #[error("SmallVec conversion err -> {0}")]
    SmallVec(#[source] core::convert::Infallible),

    #[error("Binary search not found tag: {0:?}")]
    BinaryTag(Tag),

    #[error("Binary search not found lemma: {0}")]
    BinaryLemma(String),

    #[error("Binary search not found parses: {0:?}")]
    BinaryParse(Vec<Parse>),

    #[error("No normal form in lemma {0}")]
    LostNormalForm(u64),

    #[error("No word form in lemma {0}")]
    NoForms(u64),
}

#[derive(Debug, derive_more::Display)]
pub enum Bound {
    #[display(fmt = "word_parses")]
    WordParses,
    #[display(fmt = "tags")]
    Tags,
    #[display(fmt = "lemmas")]
    Lemmas,
}

#[derive(Debug, Error)]
pub enum ParseErr {
    #[error("Index of search {idx} more than {vec} len")]
    OutOfBound { idx: u64, vec: Bound },

    #[error("Word from dictionary {0} lost his normal form")]
    LostNormal(String),

    #[error("To be continued...")]
    FutureRelease,
}
