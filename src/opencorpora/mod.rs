/// Содержит структуры для парсинга словаря из xml.
pub mod dictionary;

use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use crate::errors::{MopsErr, MopsResult};

use self::dictionary::{Lemmata, LinkTypes, Links};
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
    pub(crate) link_types: LinkTypes,
    pub(crate) links: Links,
}

impl DictionaryOpenCorpora {
    /// Инициализация слова по переданному пути.
    ///
    /// Файл читается в строку. Это быстрее, но требует больше памяти в процессе.
    pub fn init_from_path(path: &Path) -> MopsResult<Self> {
        let mut buf = String::new();
        let mut file = File::open(path).map_err(MopsErr::IO)?;
        file.read_to_string(&mut buf).map_err(MopsErr::IO)?;

        debug!("String: {}", allocative::size_of_unique(&buf));

        let dict: DictionaryOpenCorpora = from_str(&buf).map_err(MopsErr::XMLde)?;
        Ok(dict)
    }

    /// Инициализация слова по переданному пути с чтением из буфера.
    ///
    /// Чтение из буфера несколько медленнее, чем из строки, но занимает сильно меньше памяти.
    pub fn init_from_path_with_reader(path: &Path) -> MopsResult<Self> {
        let file = File::open(path).map_err(MopsErr::IO)?;
        let mut buf = BufReader::new(file);

        debug!("BufReader: {}", buf.capacity());

        let dict: DictionaryOpenCorpora =
            quick_xml::de::from_reader(&mut buf).map_err(MopsErr::XMLde)?;
        Ok(dict)
    }
}

/// Данный тест должен обязательно проверять на срабатывание,
/// что свидетельствует о том, что мы не разломали парсинг словаря.
mod test {
    #[test]
    // Парсинг текстового примера хранения
    fn test_init_test_dict() {
        let path = std::path::Path::new("data/test/test_dict.xml");
        crate::DictionaryOpenCorpora::init_from_path(path).unwrap();
    }

    #[test]
    // Парсинг текстового примера хранения c чтением через буфер
    fn test_init_test_dict_with_reader() {
        let path = std::path::Path::new("data/test/test_dict.xml");
        crate::DictionaryOpenCorpora::init_from_path_with_reader(path).unwrap();
    }

    #[ignore = "too large dictionary"]
    #[test]
    // Парсинг настоящего словаря
    fn test_init_dict() {
        let file = std::path::Path::new("dict.opcorpora.xml");

        crate::DictionaryOpenCorpora::init_from_path(file).unwrap();
    }
}

#[cfg(test)]
mod test_velocita {
    use super::*;

    #[ignore = "too large dictionary"]
    #[test]
    fn try_fastest() {
        let path = Path::new("dict.opcorpora.xml");

        let start = std::time::Instant::now();
        DictionaryOpenCorpora::init_from_path(path).unwrap();
        let end_string = start.elapsed();

        let start = std::time::Instant::now();
        DictionaryOpenCorpora::init_from_path_with_reader(path).unwrap();
        let end_buffer = start.elapsed();

        assert!(end_string < end_buffer)
    }
}
