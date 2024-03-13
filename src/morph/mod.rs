use smallvec::SmallVec;

use crate::{
    analyzer::{dictionary::LemmaDict, Tag},
    errors::DictionaryErr,
    morph::vanga::{LemmaVanga, VangaVariant},
    opencorpora::dictionary::GramWord,
};

use self::grammemes::{Grammem, Other, ParteSpeech};

/// Содержит типы хранимых граммем слов
/// в виде `unit enum`-ов для упрощения хранения.
pub mod grammemes;
/// Модуль сборки данных для Вангования
/// на основе имеющегося словаря.
pub(crate) mod vanga;

// Взято из кода Pymorphy2.
/// Непродуктивность - это в т.ч. невозможность образовывать от данных граммем префиксным-постфиксным образом новые слова.
/// Например, мы не можем образовать слово от междометия.
pub(crate) const UNPRODUCTIVE: [Grammem; 8] = [
    Grammem::ParteSpeech(ParteSpeech::Number),
    Grammem::ParteSpeech(ParteSpeech::NounPronoun),
    Grammem::ParteSpeech(ParteSpeech::Predicative),
    Grammem::ParteSpeech(ParteSpeech::Preposition),
    Grammem::ParteSpeech(ParteSpeech::Conjunction),
    Grammem::ParteSpeech(ParteSpeech::Particle),
    Grammem::ParteSpeech(ParteSpeech::Interjection),
    Grammem::Other(Other::Pronominal),
];

#[macro_export]
macro_rules! gram {
    ( $x:expr ) => {{
        $crate::morph::grammemes::ToGrammem::to_grammem($x)
    }};
}

#[macro_export]
macro_rules! grams {
    ( $($x:expr),* $(,)?  ) => {
        {
            vec![$(
                $crate::gram![$x],
            )*]
        }
    };
}

#[test]
fn test_gram() {
    let grammemes = vec![
        Grammem::ParteSpeech(ParteSpeech::Number),
        Grammem::Other(Other::Pronominal),
    ];

    let grams = grams![ParteSpeech::Number, Other::Pronominal];

    assert_eq!(gram!(ParteSpeech::Number), *grammemes.first().unwrap());
    assert_eq!(grammemes, grams);
}

impl LemmaDict {
    /// Вычленение первых граммем из леммы словаря.
    /// Первые граммемы наследуются всем остальным формам слова.
    pub(crate) fn first_tags(&self) -> Result<Tag, DictionaryErr> {
        let LemmaDict { normal_form, .. } = self;

        match &normal_form.gram {
            None => Err(DictionaryErr::LostFirstGrammemes(
                normal_form.text.to_owned(),
            )),
            Some(gram) => Ok(SmallVec::from_iter(gram.iter().map(|g| g.v))),
        }
    }

    /// Вычленение граммем остальных форм слова.
    ///
    /// Если к форме не было граммем, то вернется пустой вектор `Tag`.
    pub(crate) fn forms(forms: Vec<GramWord>) -> impl Iterator<Item = (String, Tag)> {
        forms.into_iter().map(|gram| match gram.gram {
            None => (gram.text, SmallVec::from(vec![])),
            Some(grams) => {
                let tags = SmallVec::from_iter(grams.iter().map(|gram| gram.v));
                (gram.text, tags)
            }
        })
    }
}

impl LemmaVanga {
    /// Вычленение первых граммем из леммы словаря.
    /// Первые граммемы наследуются всем остальным формам слова.
    pub(crate) fn first_tags(&self) -> Result<Tag, DictionaryErr> {
        let LemmaVanga { variants } = self;

        match variants.first() {
            None => Err(DictionaryErr::EmptyVanga),
            Some(VangaVariant { tag, .. }) => Ok(tag.to_owned()),
        }
    }

    /// Вычленение граммем остальных форм слова.
    ///
    /// Если к форме не было граммем, то вернется пустой вектор `Tag`.
    pub(crate) fn forms(forms: Vec<GramWord>) -> impl Iterator<Item = (String, Tag)> {
        forms.into_iter().map(|gram| match gram.gram {
            None => (gram.text, SmallVec::from(vec![])),
            Some(grams) => {
                let tags = SmallVec::from_iter(grams.iter().map(|gram| gram.v));
                (gram.text, tags)
            }
        })
    }
}
