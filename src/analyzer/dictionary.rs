use crate::{
    analyzer::{Lemmas, Parse, ParseTable, Tag, Tags, Vanga, SMALLLEMMA},
    errors::{DictionaryErr, MopsErr, MopsResult},
    morph::grammemes::*,
    opencorpora::{dictionary::Lemma, DictionaryOpenCorpora},
    Language,
};
use allocative::Allocative;
use fst::MapBuilder;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use smallstr::SmallString;
use std::{
    collections::{BTreeMap, HashSet},
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[derive(Debug, Default, Serialize, Deserialize, Allocative)]
/// Мета-информация словаря.
pub struct Meta {
    version: String,
    revision: u64,
    language: Language,
}

#[derive(Debug, Default, Serialize, Deserialize, Allocative)]
/// Словарь, полученный из постобработки словаря Opencorpora.
pub struct Dictionary {
    pub meta: Meta,
    pub fst: PathBuf,
    pub word_parses: ParseTable,
    #[allocative(skip)]
    pub tags: Tags,
    #[allocative(skip)]
    pub lemmas: Lemmas,
    pub paradigms: Vec<Vanga>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize)]
/// Предварительный сбор разборов слова.
pub struct ParseIntermediate {
    pub(crate) tag: Tag,
    pub(crate) form: Form,
    pub(crate) normal_form: SmallString<[u8; SMALLLEMMA]>,
}

impl Dictionary {
    /// Инициализация словаря из словаря `Opencorpor`-ы со всеми необходимыми преобразованиями и упрощениями.
    /// Словарь сохраняется двумя файлами: в fst-формате и в сериализованном виде со всеми тегами-вангами-леммами.
    pub fn init(dict: DictionaryOpenCorpora, out_dir: &Path, lang: Language) -> MopsResult<Self> {
        let mut fst = out_dir.to_path_buf();
        fst.push("dict.fst");
        let dictionary =
            Self::from_opencorpora(dict, fst.as_path(), lang).map_err(MopsErr::Dictionary)?;

        let mut dict = out_dir.to_path_buf();
        dict.push("dict.json");
        let mut writer = File::create(dict).map_err(MopsErr::IO)?;
        let bytes = serde_json::to_vec(&dictionary).map_err(MopsErr::Serde)?;
        writer.write_all(&bytes).map_err(MopsErr::IO)?;

        Ok(dictionary)
    }

    /// Открытие словаря из `dict.json` файла.
    pub fn open<P: AsRef<Path>>(path: P) -> MopsResult<Self> {
        let mut path: PathBuf = path.as_ref().into();
        path.push("dict.json");
        let mut file = File::open(&path).map_err(MopsErr::IO)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf).map_err(MopsErr::IO)?;
        let dict: Dictionary = serde_json::from_str(&buf).map_err(MopsErr::Serde)?;
        Ok(dict)
    }

    /// Преобразование словаря в нужную форму из словаря `OpenCorpora`.
    pub fn from_opencorpora(
        dict: DictionaryOpenCorpora,
        outdir: &Path,
        language: Language,
    ) -> Result<Self, DictionaryErr> {
        let DictionaryOpenCorpora {
            version,
            revision,
            mut lemmata,
            ..
        } = dict;

        let writer = File::create(outdir).map_err(DictionaryErr::IO)?;
        let wtr = std::io::BufWriter::new(writer);

        let mut fst = MapBuilder::new(wtr).map_err(DictionaryErr::FstBuild)?;

        // Предварительный сбор тегов, чтобы найти только уникальные.
        let mut tags: HashSet<Tag> = HashSet::new();

        // Предварительный сбор нормализованных слов.
        let mut lemmas: Vec<SmallString<[u8; SMALLLEMMA]>> = Vec::new();

        // Для того, чтобы добавить слова в словарь fst, нам требуется расположить их в словарном порядке.
        let mut word_map: BTreeMap<String, Vec<ParseIntermediate>> = BTreeMap::new();

        // Предварительный сбор `Vanga`-s c тегами.
        let mut paradigms = Vec::new();

        for lemma in lemmata.lemmas.iter_mut() {
            let normal = SmallString::from(lemma.normal_form.text.clone());

            lemmas.push(normal);
            lemma.collect_word_tags(lemma.id, &mut word_map, &mut tags)?;
            lemma.collect_vangas(lemma.id, &mut paradigms)?;
        }

        // Предварительный набор фиксируем в векторе, предварительно отсортировав граммемы
        let mut tags: Tags = tags.into_iter().collect_vec();
        tags.iter_mut().for_each(|e| e.sort());
        tags.sort();

        // Предварительный набор фиксируем в векторе
        let mut lemmas: Lemmas = lemmas.into_iter().collect_vec();
        lemmas.sort();

        // Финальный наборы парсингов для слов.
        let mut vec_parse: Vec<Vec<Parse>> = Vec::new();

        // После модернизации Parse, нам нужно соотнести их со словами.
        let mut word_parses: BTreeMap<String, Vec<Parse>> = BTreeMap::new();

        for (k, v) in word_map.iter_mut() {
            let mut parses = Vec::new();
            for parse_int in v {
                parse_int.tag.sort();
                let tag = parse_int.tag.to_owned();

                let parse = Parse {
                    tag: tags
                        .binary_search(&tag)
                        .map_err(|_| DictionaryErr::BinaryTag(tag.clone()))?,
                    form: parse_int.form,
                    normal_form: lemmas.binary_search(&parse_int.normal_form).map_err(|_| {
                        DictionaryErr::BinaryLemma(parse_int.normal_form.to_string())
                    })?,
                };
                parses.push(parse);
            }

            vec_parse.push(parses.clone());
            word_parses.insert(k.to_owned(), parses);
        }

        vec_parse.sort();

        for (word, tags) in word_parses.into_iter() {
            let id = vec_parse
                .binary_search(&tags)
                .map_err(|_| DictionaryErr::BinaryParse(tags))?;
            fst.insert(word, id as u64)
                .map_err(DictionaryErr::FstBuild)?;
        }

        fst.finish().map_err(DictionaryErr::FstBuild)?;
        // Образование fst закончено

        let paradigms = Vanga::parse_vangas(paradigms, &tags)?;

        Ok(Self {
            meta: Meta {
                version,
                revision,
                language,
            },
            fst: outdir.into(),
            word_parses: vec_parse,
            tags,
            lemmas,
            paradigms,
        })
    }
}

impl Lemma {
    /// Сборка наборов тегов для каждого представлено слова в лемме.
    ///
    /// У слова может быть не один набор тегов. Например, `cтали` - сущ. в род.пад. **и** гл. мн.ч. в прош.вр.
    fn collect_word_tags(
        &self,
        id: u64,
        words: &mut BTreeMap<String, Vec<ParseIntermediate>>,
        tags: &mut HashSet<Tag>,
    ) -> Result<(), DictionaryErr> {
        // В начальной форме содержатся общие для леммы граммемы, как одушевленность, часть речи и т.п.
        // Эти теги должны быть вычленены и прокинуты всем словам.
        let inizio_grammemes = self.normal_tags()?;

        match &self.forms.as_ref().filter(|v| !v.is_empty()) {
            None => {
                // Если у нас нет других форм, кроме начальной, мы сохраняем просто начальную.
                let normal_form = &self.normal_form.text;
                let form = Form::Word(FWord::Normal(self.id));

                Self::to_word_tags(
                    inizio_grammemes,
                    words,
                    tags,
                    normal_form,
                    normal_form,
                    form,
                )
            }
            Some(forms) => {
                let mut iter = Self::forms(forms.to_owned().to_owned());
                let normal_form = &self.normal_form.text;

                // Первая форма аналогична начальной, поэтому мы просто совмещаем теги и сохраняем за первой формой FWord // todo: FormInizio release 0.1.1
                {
                    let (first, mut first_grammemes) =
                        iter.next().ok_or(DictionaryErr::NoForms(id))?;
                    first_grammemes.extend(inizio_grammemes.clone());
                    let form = Form::Word(FWord::Normal(self.id));

                    Self::to_word_tags(first_grammemes, words, tags, &first, normal_form, form)
                }

                for (text, mut grammemes) in iter {
                    grammemes.extend(inizio_grammemes.clone());
                    let form = Form::Word(FWord::Different(self.id));

                    Self::to_word_tags(grammemes, words, tags, &text, normal_form, form)
                }
            }
        }

        Ok(())
    }

    /// Добавление в хэшмапу формы слова и набора тегов для нее.
    /// Если слово уже содержится в хэшмапе, добавление имеющегося набора тегов в вектор наборов этого слова.
    fn to_word_tags(
        grammemes: Tag,
        words: &mut BTreeMap<String, Vec<ParseIntermediate>>,
        tags: &mut HashSet<Tag>,
        word: &str,
        normal_form: &str,
        form: Form,
    ) {
        tags.insert(grammemes.clone());
        let parses = words.entry(word.to_lowercase()).or_default();

        parses.push(ParseIntermediate {
            tag: grammemes,
            form,
            normal_form: normal_form.into(),
        })
    }
}

/// Нахождение максимально длинной основы слова.
pub fn longest_common_substring(mut data: Vec<String>) -> String {
    // Пока что мы игнорируем букву ё todo
    data.iter_mut().for_each(|w| {
        if w.contains('ё') {
            *w = w.replace('ё', "е");
        }
    });

    match data.len() {
        0 => String::new(),
        1 => data[0].clone(),
        _ => {
            let base = &data[0];

            // declare tracking vars to walk through the vector and track best match
            let mut sub_string = String::new();
            let mut best_match = String::new();

            for char in base.chars() {
                sub_string.push(char);

                for word in &data[1..] {
                    if word.contains(&sub_string) {
                        if sub_string.len() > best_match.len() {
                            best_match = sub_string.clone();
                        }
                    } else {
                        if sub_string.len() == best_match.len() && sub_string.contains(&best_match)
                        {
                            best_match.pop();
                        }

                        sub_string.clear();
                        break;
                    }
                }
            }

            best_match
        }
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use crate::{
        opencorpora::dictionary::{Gram, NormalForm},
        DictionaryOpenCorpora,
    };
    use test_case::test_case;

    /// Создание тестового словаря + fst для проверки функций.
    pub(crate) fn make_dict(file_path: &str) -> Dictionary {
        let file = Path::new(file_path);
        let fst = "data/test/test_lemmas_result.fst".to_string();
        let fst = Path::new(&fst);
        File::create(fst).unwrap();

        let dict = DictionaryOpenCorpora::init_from_path(file).unwrap();
        Dictionary::from_opencorpora(dict, fst, Language::Russian).unwrap()
    }

    #[test_case(vec!["apricot", "rice", "cricket"] => "ric")]
    #[test_case(vec!["apricot", "banana"] => "a")]
    #[test_case(vec!["foo", "bar", "baz"] => "")]
    #[test_case(vec!["ёж", "ежа", "ежу", "ежом"] => "еж")]
    #[test_case(vec!["ёжистее", "ёжистее", "ёжистей", "поёжистее", "поёжистей"] => "ежисте")]
    fn test_longest_substing(slice: Vec<&str>) -> String {
        let slice = slice.into_iter().map(|s| s.to_string()).collect_vec();
        longest_common_substring(slice)
    }

    #[test_case(Lemma { id: 0, normal_form: NormalForm { text: "ёж".to_owned(), gram: Some(vec![Gram { v: Grammem::ParteSpeech(ParteSpeech::Noun) }]) }, forms: None }, vec![Grammem::ParteSpeech(ParteSpeech::Noun)].into())]
    fn test_normal_tags(lemma: Lemma, tag: Tag) {
        assert_eq!(lemma.normal_tags().unwrap(), tag);
    }

    #[test]
    /// Иногда в формах леммы нет никаких дополнительных граммем.
    /// Тест-проверка на то, что такие формы не пропадают, а имеют только граммем начальной формы.
    fn test_no_form() {
        let dict = make_dict("data/test/piccolo_dict.xml");
        assert_eq!(dict.word_parses.len(), 4usize);
        std::fs::remove_file(dict.fst).unwrap();
    }
}
