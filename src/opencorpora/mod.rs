/// Содержит структуры для парсинга словаря Opencorpora из xml.
pub(crate) mod dictionary;

use std::{fs::File, io::BufReader, path::Path};

use self::dictionary::{Lemmata, Links};
use crate::errors::{MopsErr, MopsResult};
use quick_xml::de::from_str;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct DictionaryOpenCorpora {
    #[serde(rename = "@version")]
    pub(crate) version: String,
    #[serde(rename = "@revision")]
    pub(crate) revision: u64,

    pub(crate) lemmata: Lemmata,
    pub(crate) links: Links,
}

impl DictionaryOpenCorpora {
    /// Инициализация слова по переданному пути.
    ///
    /// Файл читается в строку. Это быстрее, но требует больше памяти в процессе.
    pub fn init_from_path<P: AsRef<Path>>(path: P) -> MopsResult<Self> {
        let buf = std::fs::read_to_string(path).map_err(MopsErr::IO)?;

        debug!("String: {}", allocative::size_of_unique(&buf));

        let dict: DictionaryOpenCorpora = from_str(&buf).map_err(MopsErr::XMLde)?;
        Ok(dict)
    }

    /// Инициализация слова по переданному пути с чтением из буфера.
    ///
    /// Чтение из буфера несколько медленнее, чем из строки, но занимает сильно меньше памяти.
    pub fn init_from_path_with_reader<P: AsRef<Path>>(path: P) -> MopsResult<Self> {
        let file = File::open(&path).map_err(|error| MopsErr::File {
            file: path.as_ref().into(),
            error,
        })?;
        let mut buf = BufReader::new(file);

        debug!("BufReader: {}", buf.capacity());

        let dict: DictionaryOpenCorpora =
            quick_xml::de::from_reader(&mut buf).map_err(MopsErr::XMLde)?;
        Ok(dict)
    }
}

mod test_parse {
    #[test]
    /// Парсинг текстового примера хранения.
    ///
    /// Данный тест должен обязательно проверять на срабатывание,
    /// что свидетельствует о том, что мы не разломали парсинг словаря.
    fn test_init_test_dict() {
        crate::DictionaryOpenCorpora::init_from_path("data/test/test_dict.xml").unwrap();
    }

    #[ignore = "Too large dictionary"]
    #[test]
    // Парсинг настоящего словаря
    fn test_init_dict() {
        crate::DictionaryOpenCorpora::init_from_path("dict.opcorpora.xml").unwrap();
    }
}

#[cfg(test)]
mod test_features {
    use super::*;

    #[ignore = "Too large dictionary"]
    #[test]
    /// Проверка, что инициализация из строки меньше, чем инициализация через reader.
    fn try_fastest() {
        let start = std::time::Instant::now();
        DictionaryOpenCorpora::init_from_path("dict.opcorpora.xml").unwrap();
        let end_string = start.elapsed();

        let start = std::time::Instant::now();
        DictionaryOpenCorpora::init_from_path_with_reader("dict.opcorpora.xml").unwrap();
        let end_buffer = start.elapsed();

        assert!(end_string < end_buffer)
    }
}
