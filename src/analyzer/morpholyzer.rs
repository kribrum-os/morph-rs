use fst::{IntoStreamer, Streamer};
use itertools::Itertools;
use tracing::{debug, error};

use crate::{
    analyzer::{declension::alphabet_stream, Parse, WordForm},
    errors::ParseErr,
    morph::grammemes::Grammem,
    InflectWord, Method, MorphAnalyzer, NormalizedWord, ParsedWord,
};
use std::collections::HashMap;

use super::InflectWords;

impl MorphAnalyzer {
    /// Преобразование разбора слова в соответствующую структуру.
    pub(crate) fn try_into_parse(&self, word: &str, parse: &Parse) -> Result<ParsedWord, ParseErr> {
        Ok(ParsedWord {
            word: word.to_string(),
            tags: self.get_tag(parse.tag)?.to_owned(),
            normal_form: self.get_lemmas(parse.normal_form)?.to_string(),
            method: Method::Dictionary,
        })
    }

    /// Преобразование разбора слова в нормализованное слово.
    pub(crate) fn try_into_normalized(&self, parse: &Parse) -> Result<NormalizedWord, ParseErr> {
        Ok(NormalizedWord {
            normal_word: self.get_lemmas(parse.normal_form)?.to_string(),
            tags: self.get_tag(parse.tag)?.to_owned(),
            method: Method::Dictionary,
        })
    }

    /// Преобразование разбора слова в измененную форму слова.
    pub(crate) fn try_into_inflect(
        &self,
        word: String,
        parse: &Parse,
    ) -> Result<InflectWord, ParseErr> {
        Ok(InflectWord {
            inflect_form: word,
            tags: self.get_tag(parse.tag)?.to_owned(),
            normal_form: self.get_lemmas(parse.normal_form)?.to_string(),
            method: Method::Dictionary,
        })
    }

    /// Преобразование разбора слова в измененную форму слова
    /// с учетом уже найденных данных.
    pub(crate) fn try_into_inflect_hint(
        &self,
        word: String,
        word_form: &WordForm,
    ) -> Result<InflectWord, ParseErr> {
        Ok(InflectWord {
            inflect_form: word,
            tags: word_form.tag.to_owned(),
            normal_form: word_form.lemma.to_string(),
            method: Method::Dictionary,
        })
    }
}

impl MorphAnalyzer {
    /// Фильтрация нужных `Parse` слов(а) в зависимости от запроса граммем.
    /// Возвращается итератор вместе с `ParseId`, индексом в `fst::Map`.
    pub fn id_forms<'a>(
        &'a self,
        word: &'a str,
        ids: &'a [u32],
        word_id: Option<u64>,
        grammemes: &'a Option<Vec<Grammem>>,
    ) -> impl Iterator<Item = (u64, &'a Parse)> {
        self.word_parses.iter().enumerate().flat_map(move |(i, p)| {
            p.iter()
                .filter_map(|p| {
                    let id = p.form.id();

                    if id.is_none() {
                        error!("{}", ParseErr::LostLemmaId(word.into()));
                        return None;
                    }

                    // Unwrap() на id безопасен в связи с вышеизложенной проверкой.
                    match (grammemes.is_none(), word_id) {
                        (true, Some(word_id)) => {
                            // Начальная форма может совпадать с нормальной.
                            if word_id == id.unwrap() && (p.form.is_inizio() | p.form.is_normal()) {
                                Some((i as u64, p))
                            } else {
                                None
                            }
                        }
                        // Если специально граммемы не определены, нам нужно вернуть все варианты парсингов
                        // Если функция вызывалась из `inflect()`, а не `declension()`, сортировка будет в
                        // следующих звеньях логической цепочки
                        _ => {
                            if ids.contains(&(id.unwrap() as u32)) {
                                Some((i as u64, p))
                            } else {
                                None
                            }
                        }
                    }
                })
                .collect_vec()
        })
    }

    /// Сбор префикс-ограничений для `fst::Stream`.
    ///
    /// Префикс-ограничения - это пара префиксов, внутри которых Streamer проходит fst-словарь,
    /// игнорируя все остальные части словаря.
    pub(crate) fn collect_stream_hashset<'a>(
        &'a self,
        word: &str,
        grammemes: &Option<Vec<Grammem>>,
        id_forms: impl Iterator<Item = (u64, &'a Parse)>,
        hash_set: &mut HashMap<(String, Option<String>), Vec<WordForm<'a>>>,
    ) -> Result<(), ParseErr> {
        for (i, parse) in id_forms {
            let tag = self.get_tag(parse.tag)?;

            if let Some(grammemes) = grammemes.as_ref() {
                if !grammemes.iter().all(|item| tag.contains(item)) {
                    continue;
                };
            }

            let normal_form = self.get_lemmas(parse.normal_form)?;

            for (first, last) in
                alphabet_stream(word, normal_form, tag.to_owned()).map_err(ParseErr::Declension)?
            {
                let word_form = WordForm {
                    i,
                    tag,
                    lemma: normal_form,
                };

                let vec = hash_set.entry((first, last)).or_default();
                if !vec.contains(&word_form) {
                    vec.push(word_form)
                }
            }
        }

        Ok(())
    }

    /// Итерация по fst::Stream с учетом префиксных ограничений для сокращения прохода.
    /// При итерации в `InflectWords` сохраняются только те формы,
    /// которые соответствуют индексу в fst::Map -> WordForm { i, ..}.
    pub(crate) fn iter_fst(
        &self,
        hash_set: &mut HashMap<(String, Option<String>), Vec<WordForm<'_>>>,
        inflect: &mut InflectWords,
    ) -> Result<(), ParseErr> {
        let map = &self.fst;

        for ((first, last), vec) in hash_set.iter() {
            debug!("{first}-{last:?}");

            let range = match last {
                Some(last) => map.range().ge(first).lt(last),
                None => map.range().ge(first).le(first),
            };
            let mut stream = range.into_stream();

            while let Some((key, value)) = stream.next() {
                for word_form in vec.iter().filter(|WordForm { i, .. }| *i == value) {
                    debug!("Value == i was found");
                    let inflect_word = self.try_into_inflect_hint(
                        String::from_utf8_lossy(key).to_string(),
                        word_form,
                    )?;
                    if !inflect.0.contains(&inflect_word) {
                        inflect.0.push(inflect_word);
                    }
                }
            }
        }

        Ok(())
    }
}
