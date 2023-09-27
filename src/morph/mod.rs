use smallvec::SmallVec;

use crate::{
    analyzer::Tag,
    errors::DictionaryErr,
    opencorpora::dictionary::{GramWord, Lemma},
};

/// Содержит типы хранимых граммем слов
/// в виде `unit enum`-ов для упрощения хранения.
pub mod grammemes;

impl Lemma {
    /// Вычленение тегов нормализованной формы слова.
    pub(crate) fn normal_tags(&self) -> Result<Tag, DictionaryErr> {
        let Lemma {
            id, normal_form, ..
        } = self;

        match &normal_form.gram {
            None => Err(DictionaryErr::LostNormalForm(*id)),
            Some(gram) => Ok(SmallVec::from_iter(gram.iter().map(|g| g.v))),
        }
    }

    /// Вычленение тегов остальных форм слова.
    pub(crate) fn forms(forms: Vec<GramWord>) -> impl Iterator<Item = (String, Tag)> {
        forms.into_iter().map(|gram| match gram.gram {
            None => (gram.text, Tag::from(vec![])),
            Some(grams) => {
                let tags = SmallVec::from_iter(grams.iter().map(|gram| gram.v));
                (gram.text, tags)
            }
        })
    }

    /// Вычленение всех форм слова для использования в стемминге (определении общего корня).
    pub(crate) fn to_longest_common_substring(&self) -> Vec<String> {
        let Lemma {
            id: _,
            normal_form,
            forms,
        } = self.clone();

        if let Some(forms) = forms {
            let len = forms.len() + 1;
            let mut vec = Vec::with_capacity(len);
            vec.push(normal_form.text);

            for word in forms {
                vec.push(word.text);
            }

            vec
        } else {
            vec![normal_form.text]
        }
    }
}
