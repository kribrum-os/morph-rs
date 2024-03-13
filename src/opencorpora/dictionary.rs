use crate::morph::grammemes::Grammem;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
/// Все `<lemmata>` из словаря.
pub struct Lemmata {
    #[serde(rename = "$value")]
    pub(crate) lemmas: Vec<Lemma>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
/// Леммы словаря.
pub struct Lemma {
    #[serde(rename = "@id")]
    pub(crate) id: u64,
    /// Разные грамматические формы леммы под конкретными id. Начальная форма через "l", спряжения через "f".
    #[serde(rename = "l")]
    pub(crate) normal_form: NormalForm,
    #[serde(rename = "f")]
    pub(crate) forms: Option<Vec<GramWord>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub(crate) struct NormalForm {
    #[serde(rename = "@t")]
    pub(crate) text: String,
    #[serde(rename = "$value")]
    pub(crate) gram: Option<Vec<Gram>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub(crate) struct GramWord {
    #[serde(rename = "@t")]
    pub(crate) text: String,
    #[serde(rename = "$value")]
    pub(crate) gram: Option<Vec<Gram>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub(crate) struct Gram {
    #[serde(rename = "@v")]
    pub(crate) v: Grammem,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Links {
    #[serde(rename = "$value")]
    pub links: Vec<Link>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Eq, Hash)]
pub struct Link {
    #[serde(rename = "@type")]
    pub(crate) type_id: u64,
    #[serde(rename = "@from")]
    pub(crate) lemma_id: u64,
    #[serde(rename = "@to")]
    pub(crate) variant: u64,
}

impl PartialOrd for Lemma {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.id.cmp(&other.id))
    }
}

impl Ord for Lemma {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}
