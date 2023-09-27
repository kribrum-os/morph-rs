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

/// Далее идут грамматические связи между словами. Что с ними делать - рассмотрим позже.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct LinkTypes {
    #[serde(rename = "$value")]
    pub links: Vec<LinkType>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename = "type")]
pub struct LinkType {
    #[serde(rename = "$text")]
    link_type: LinkConnotation,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Links {
    #[serde(rename = "$value")]
    pub links: Vec<Link>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Link {
    #[serde(rename = "@type")]
    pub(crate) type_id: u64,
    #[serde(rename = "@from")]
    pub(crate) from: u64,
    #[serde(rename = "@to")]
    pub(crate) to: u64,
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[allow(clippy::upper_case_acronyms)]
#[allow(non_camel_case_types)]
pub enum LinkConnotation {
    #[serde(rename = "ADJF-ADJS")]
    ADJF_ADJS,
    #[serde(rename = "ADJF-COMP")]
    ADJF_COMP,
    #[serde(rename = "INFN-VERB")]
    INFN_VERB,
    #[serde(rename = "INFN-PRTF")]
    INFN_PRTF,
    #[serde(rename = "INFN-GRND")]
    INFN_GRND,
    #[serde(rename = "PRTF-PRTS")]
    PRTF_PRTS,
    #[serde(rename = "NAME-PATR")]
    NAME_PATR,
    #[serde(rename = "PATR_MASC-PATR_FEMN")]
    PATR_MASC_PATR_FEMN,
    #[serde(rename = "SURN_MASC-SURN_FEMN")]
    SURN_MASC_SURN_FEMN,
    #[serde(rename = "SURN_MASC-SURN_PLUR")]
    SURN_MASC_SURN_PLUR,
    #[serde(rename = "PERF-IMPF")]
    PERF_IMPF,
    #[serde(rename = "ADJF-SUPR_ejsh")]
    ADJF_SUPR_ejsh,
    #[serde(rename = "PATR_MASC_FORM-PATR_MASC_INFR")]
    PATR_MASC_FORM_PATR_MASC_INFR,
    #[serde(rename = "PATR_FEMN_FORM-PATR_FEMN_INFR")]
    PATR_FEMN_FORM_PATR_FEMN_INFR,
    #[serde(rename = "ADJF_eish-SUPR_nai_eish")]
    ADJF_eish_SUPR_nai_eish,
    #[serde(rename = "ADJF-SUPR_ajsh")]
    ADJF_SUPR_ajsh,
    #[serde(rename = "ADJF_aish-SUPR_nai_aish")]
    ADJF_aish_SUPR_nai_aish,
    #[serde(rename = "ADJF-SUPR_suppl")]
    ADJF_SUPR_suppl,
    #[serde(rename = "ADJF-SUPR_nai")]
    ADJF_SUPR_nai,
    #[serde(rename = "ADJF-SUPR_slng")]
    ADJF_SUPR_slng,
    #[serde(rename = "FULL-CONTRACTED")]
    FULL_CONTRACTED,
    #[serde(rename = "NORM-ORPHOVAR")]
    NORM_ORPHOVAR,
    #[serde(rename = "CARDINAL-ORDINAL")]
    CARDINAL_ORDINAL,
    #[serde(rename = "SBST_MASC-SBST_FEMN")]
    SBST_MASC_SBST_FEMN,
    #[serde(rename = "SBST_MASC-SBST_PLUR")]
    SBST_MASC_SBST_PLUR,
    #[serde(rename = "ADVB-COMP")]
    ADVB_COMP,
    #[serde(rename = "ADJF_TEXT-ADJF_NUMBER")]
    ADJF_TEXT_ADJF_NUMBER,
}
