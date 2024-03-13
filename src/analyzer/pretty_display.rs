use smallstr::SmallString;

use super::{InflectWords, NormalizedWords, Parse, ParsedWords, Tag};
use crate::{
    errors::{Bound, ParseErr},
    InflectWord, MorphAnalyzer, NormalizedWord, ParsedWord, SMALLLEMMA,
};

impl MorphAnalyzer {
    pub(crate) fn get_tag(&self, index: usize) -> Result<&Tag, ParseErr> {
        self.tags.get(index).ok_or(ParseErr::OutOfBound {
            idx: index as u64,
            vec: Bound::Tags,
        })
    }

    pub(crate) fn get_lemmas(
        &self,
        index: usize,
    ) -> Result<&SmallString<[u8; SMALLLEMMA]>, ParseErr> {
        self.lemmas.get(index).ok_or(ParseErr::OutOfBound {
            idx: index as u64,
            vec: Bound::Lemmas,
        })
    }

    pub(crate) fn get_parse(&self, idx: u64) -> Result<&Vec<Parse>, ParseErr> {
        self.word_parses
            .get(idx as usize)
            .ok_or(ParseErr::OutOfBound {
                idx,
                vec: Bound::WordParses,
            })
    }

    pub(crate) fn get_row_id(&self, index: usize) -> Result<&Vec<u32>, ParseErr> {
        self.lemmas_rows.get(index).ok_or(ParseErr::OutOfBound {
            idx: index as u64,
            vec: Bound::LemmasRow,
        })
    }
}

impl std::fmt::Display for ParsedWord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParsedWord = '{}', ", self.word)?;
        write!(f, "tags: [")?;
        let iter = self.tags.iter();
        let len = iter.clone().count();
        let last = iter.clone().last();
        if len > 1 {
            for tag in iter.take(len - 1) {
                write!(f, "{}, ", tag)?;
            }
        }
        match last {
            Some(last) => write!(f, "{}", last)?,
            None => write!(f, "")?,
        };
        write!(f, "], ")?;
        write!(f, "normal_form = '{}', ", self.normal_form)?;
        write!(f, "Method::{}", self.method)
    }
}

impl std::fmt::Display for ParsedWords {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let len = self.0.len();
        let iter = self.0.iter();
        let last = iter.clone().last();
        if len > 1 {
            for parsed in iter.take(len - 1) {
                writeln!(f, "{},", parsed)?;
            }
        }
        match last {
            Some(last) => write!(f, "{}", last),
            None => write!(f, ""),
        }
    }
}

impl std::fmt::Display for NormalizedWord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NormalizedWord = '{}', ", self.normal_word)?;
        write!(f, "tags: [")?;
        let iter = self.tags.iter();
        let len = iter.clone().count();
        let last = iter.clone().last();
        if len > 1 {
            for tag in iter.take(len - 1) {
                write!(f, "{}, ", tag)?;
            }
        }
        match last {
            Some(last) => write!(f, "{}", last)?,
            None => write!(f, "")?,
        };
        write!(f, "], ")?;
        write!(f, "Method::{}", self.method)
    }
}

impl std::fmt::Display for NormalizedWords {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let len = self.0.len();
        let iter = self.0.iter();
        let last = iter.clone().last();
        if len > 1 {
            for normalized in iter.take(len - 1) {
                writeln!(f, "{},", normalized)?;
            }
        }
        match last {
            Some(last) => write!(f, "{}", last),
            None => write!(f, ""),
        }
    }
}

impl std::fmt::Display for InflectWord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InflectForm = '{}', ", self.inflect_form)?;
        write!(f, "tags: [")?;
        let iter = self.tags.iter();
        let len = iter.clone().count();
        let last = iter.clone().last();
        if len > 1 {
            for tag in iter.take(len - 1) {
                write!(f, "{}, ", tag)?;
            }
        }
        match last {
            Some(last) => write!(f, "{}", last)?,
            None => write!(f, "")?,
        };
        write!(f, "], ")?;
        write!(f, "normal_form = '{}', ", self.normal_form)?;
        write!(f, "Method::{}", self.method)
    }
}

impl std::fmt::Display for InflectWords {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let len = self.0.len();
        let iter = self.0.iter().enumerate();
        let last = iter.clone().last();
        if len > 1 {
            for (i, inflect) in iter.take(len - 1) {
                writeln!(f, "Parse {i}: {},", inflect)?;
            }
        }
        match last {
            Some((i, last)) => write!(f, "Parse {i}: {}", last),
            None => write!(f, ""),
        }
    }
}

#[cfg(test)]
mod test {
    use smallvec::SmallVec;

    use crate::{
        analyzer::{Method::Dictionary, ParsedWords},
        grams,
        morph::grammemes::*,
        ParsedWord,
    };

    #[test]
    fn test_display() {
        let result = "ParsedWord = 'москве', tags: [Noun, Inanimate, Locativus, Feminine, Singular, SingulariaTantum, Geography], normal_form = 'москва', Method::Dictionary,\nParsedWord = 'москве', tags: [Noun, Inanimate, Dativus, Feminine, Singular, SingulariaTantum, Geography], normal_form = 'москва', Method::Dictionary";

        let parses = ParsedWords(vec![
            ParsedWord {
                word: "москве".to_string(),
                tags: SmallVec::from(grams![
                    ParteSpeech::Noun,
                    Animacy::Inanimate,
                    Case::Locativus,
                    Gender::Feminine,
                    Number::Singular,
                    Number::SingulariaTantum,
                    Other::Geography
                ]),
                normal_form: "москва".to_string(),
                method: Dictionary,
            },
            ParsedWord {
                word: "москве".to_string(),
                tags: SmallVec::from(grams![
                    ParteSpeech::Noun,
                    Animacy::Inanimate,
                    Case::Dativus,
                    Gender::Feminine,
                    Number::Singular,
                    Number::SingulariaTantum,
                    Other::Geography
                ]),
                normal_form: "москва".to_string(),
                method: Dictionary,
            },
        ]);
        assert_eq!(parses.to_string(), result);
    }
}
