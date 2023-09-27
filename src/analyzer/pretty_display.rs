use super::{NormalizedWords, ParsedWords};
use crate::{NormalizedWord, ParsedWord};

impl std::fmt::Display for ParsedWord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParsedWord = '{}', ", self.word)?;
        write!(f, "tags: [")?;
        let iter = self.tags.iter();
        let len = iter.clone().count();
        let last = iter.clone().last();
        if len > 1 {
            for tag in iter.take(len - 2) {
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
            for parsed in iter.take(len - 2) {
                write!(f, "{}, ", parsed)?;
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
                write!(f, "{}, ", normalized)?;
            }
        }
        match last {
            Some(last) => write!(f, "{}", last),
            None => write!(f, ""),
        }
    }
}
