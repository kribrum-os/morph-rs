use crate::{
    analyzer::{Lemmas, Parse, ParseTable, Tag, Tags, Vanga, SMALLLEMMA},
    errors::{Cycle, DictionaryErr, MopsErr, MopsResult},
    morph::{grammemes::*, vanga::LemmaVanga},
    opencorpora::{
        dictionary::{GramWord, Link, Links, NormalForm},
        DictionaryOpenCorpora,
    },
    Language,
};
use allocative::Allocative;
use fst::MapBuilder;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use smallstr::SmallString;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use super::{LemmasRows, OpCLid};

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
    pub word_parses: ParseTable,
    #[allocative(skip)]
    pub tags: Tags,
    #[allocative(skip)]
    pub lemmas: Lemmas,
    pub paradigms: Vec<Vanga>,
    pub lemmas_rows: LemmasRows,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize)]
/// Предварительный сбор разборов слова.
pub struct ParseIntermediate {
    pub(crate) tag: Tag,
    pub(crate) form: Form,
    pub(crate) normal_form: SmallString<[u8; SMALLLEMMA]>,
    pub(crate) opcorp_lemma: Vec<OpCLid>,
}

impl Dictionary {
    /// Инициализация словаря из словаря `Opencorpor`-ы со всеми необходимыми преобразованиями и упрощениями.
    /// Словарь сохраняется двумя файлами: в fst-формате и в сериализованном виде со всеми тегами-вангами-леммами.
    pub fn init<P: AsRef<Path>>(
        dict: DictionaryOpenCorpora,
        out_dir: P,
        lang: Language,
    ) -> MopsResult<Self> {
        let fst = out_dir.as_ref().join("dict.fst");
        let dictionary =
            Self::from_opencorpora(dict, fst.as_path(), lang).map_err(MopsErr::Dictionary)?;

        let dict = out_dir.as_ref().join("dict.json");

        let mut writer = File::create(dict).map_err(MopsErr::IO)?;

        let bytes = serde_json::to_vec(&dictionary).map_err(MopsErr::Serde)?;
        writer.write_all(&bytes).map_err(MopsErr::IO)?;

        Ok(dictionary)
    }

    /// Открытие словаря из `dict.json` файла, используя Reader для файла.
    ///
    /// `WARN!` ОЧЕНЬ долгий процесс чтения.
    /// Рекомендуется использование только при очень ограниченной оперативной памяти.
    pub fn open_from_reader<P: AsRef<Path>>(path: P) -> MopsResult<Self> {
        let path = path.as_ref().join("dict.json");
        let reader = File::open(&path).map_err(|error| MopsErr::File { file: path, error })?;
        let dict: Dictionary = serde_json::from_reader(&reader).map_err(MopsErr::Serde)?;

        Ok(dict)
    }

    /// Открытие словаря из `dict.json` файла.
    pub fn open<P: AsRef<Path>>(path: P) -> MopsResult<Self> {
        let path: PathBuf = path.as_ref().join("dict.json");
        let buf = std::fs::read_to_string(path).map_err(MopsErr::IO)?;
        let dict: Dictionary = serde_json::from_str(&buf).map_err(MopsErr::Serde)?;
        Ok(dict)
    }

    /// Преобразование словаря в нужную форму из словаря `Opencorpora`.
    pub fn from_opencorpora<P: AsRef<Path>>(
        dict: DictionaryOpenCorpora,
        outdir: P,
        language: Language,
    ) -> Result<Self, DictionaryErr> {
        let DictionaryOpenCorpora {
            version,
            revision,
            lemmata,
            links,
        } = dict;

        let link_connotation: HashMap<u64, Vec<u64>> = links.collect_lemmas();

        let writer = File::create(&outdir).map_err(|error| DictionaryErr::Outdir {
            outdir: outdir.as_ref().into(),
            error,
        })?;
        let wtr = std::io::BufWriter::new(writer);

        let mut fst = MapBuilder::new(wtr).map_err(DictionaryErr::FstBuild)?;

        // Предварительный сбор тегов, чтобы найти только уникальные.
        let mut tags: HashSet<Tag> = HashSet::new();

        // Предварительный сбор нормализованных слов.
        let mut lemmas: Vec<SmallString<[u8; SMALLLEMMA]>> = Vec::new();

        // Для того, чтобы добавить слова в словарь fst, нам требуется расположить их в словарном порядке.
        let mut word_map: BTreeMap<String, Vec<ParseIntermediate>> = BTreeMap::new();

        // Предварительный сбор `Vanga`-s c тегами.
        let mut paradigms = HashMap::new();

        // Сбор всех LemmaId, чтобы отсеять впоследствии id, участвующие в LinkTypes (link_connotation)
        // и пройтись по ни с чем другим не связанным леммам.
        let mut all_lemmas = std::collections::BTreeSet::new();

        // Сбор всех id леммы из Opencorpora, относящихся к слову. После полной нормализации это
        // необходимо, чтобы найти все формы слова (в т.ч. не из той же леммы).
        let mut lemmas_rows = LemmasRows::default();

        let mut lemmata_map = HashMap::new();
        for lemma in lemmata.lemmas {
            all_lemmas.insert(lemma.id);
            lemmata_map.insert(
                lemma.id,
                LemmaDict {
                    normal_form: lemma.normal_form,
                    variants: lemma.forms,
                },
            );
        }

        for (lemma_id, variants) in link_connotation {
            let mut lemma_row: Vec<OpCLid> = Vec::with_capacity(1 + variants.len());
            lemma_row.push(lemma_id as u32);
            lemma_row.extend(variants.iter().map(|v| *v as u32).collect_vec());
            lemma_row.sort();

            let mut vangas_words = Vec::new();

            let normal = lemmata_map
                .get(&lemma_id)
                .ok_or(DictionaryErr::LostLemmaId(lemma_id, Cycle::Normal))?;
            all_lemmas.remove(&lemma_id);

            let normal_form = normal.normal_form.text.clone();
            lemmas.push(SmallString::from_str(&normal_form));

            normal.collect_word_tags(
                lemma_id,
                Lemmatization::Normal,
                None,
                &mut word_map,
                &mut tags,
                lemma_row.clone(),
                &mut vangas_words,
            )?;
            let mut lemma_vanga = LemmaVanga::push_normal(normal)?;

            for variant_id in variants {
                let lemma = lemmata_map
                    .get(&variant_id)
                    .ok_or(DictionaryErr::LostLemmaId(variant_id, Cycle::Variant))?;
                all_lemmas.remove(&variant_id);

                lemma.collect_word_tags(
                    variant_id,
                    Lemmatization::Inizio,
                    Some(normal_form.clone()),
                    &mut word_map,
                    &mut tags,
                    lemma_row.clone(),
                    &mut vangas_words,
                )?;
                lemma_vanga.update_form(lemma.to_owned())?;
            }

            lemmas_rows.push(lemma_row);
            lemma_vanga.collect_vangas(&mut paradigms, vangas_words)?;
        }

        // Оставшиеся, не участвующие в link_connotations, леммы также нужно обработать.
        for lost_id in all_lemmas {
            let lemma = lemmata_map
                .get(&lost_id)
                .ok_or(DictionaryErr::LostLemmaId(lost_id, Cycle::Lost))?;

            let mut vangas_words = Vec::new();

            let normal_form = lemma.normal_form.text.clone();
            lemmas.push(SmallString::from_str(&normal_form));

            lemma.collect_word_tags(
                lost_id,
                Lemmatization::Normal,
                None,
                &mut word_map,
                &mut tags,
                vec![lost_id as u32],
                &mut vangas_words,
            )?;
            let lemma_vanga = LemmaVanga::push_normal(lemma)?;

            lemmas_rows.push(vec![lost_id as u32]);
            lemma_vanga.collect_vangas(&mut paradigms, vangas_words)?;
        }

        // Предварительный набор фиксируем в векторе, предварительно отсортировав граммемы
        let mut tags: Tags = tags.into_iter().collect_vec();
        tags.iter_mut().for_each(|e| e.sort());
        tags.sort();

        // Предварительный набор фиксируем в векторе
        let mut lemmas: Lemmas = lemmas.into_iter().collect_vec();
        lemmas.sort();

        lemmas_rows.sort();

        // Финальные наборы парсингов для слов.
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
                    lemma_row_id: lemmas_rows
                        .binary_search(&parse_int.opcorp_lemma)
                        .map_err(|_| DictionaryErr::BinaryRow(parse_int.opcorp_lemma.clone()))?,
                };
                parses.push(parse);
            }

            parses.sort();
            vec_parse.push(parses.clone());
            word_parses.insert(k.to_owned(), parses.clone());

            if k.contains('ё') {
                let k = k.replace('ё', "е");
                word_parses.insert(k.to_owned(), parses);
            }
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
            word_parses: vec_parse,
            tags,
            lemmas,
            paradigms,
            lemmas_rows,
        })
    }
}

pub enum Lemmatization {
    Normal,
    Inizio,
}

#[derive(Clone)]
pub struct LemmaDict {
    pub(crate) normal_form: NormalForm,
    pub(crate) variants: Option<Vec<GramWord>>,
}

impl LemmaDict {
    /// Сборка наборов тегов для каждого представленного слова в лемме.
    ///
    /// У слова может быть не один набор тегов. Например, `cтали` - сущ. в род.пад. **и** гл. мн.ч. в прош.вр.
    ///
    /// Normalization передается тогда, когда слово не является нормальной (лемматизированной) формой.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn collect_word_tags(
        &self,
        lemma_id: u64,
        // С чего отсчитывается лемма
        first: Lemmatization,
        // Если слово - не нормальная форма (лемма), то необходимо эту лемму передать.
        normalization: Option<String>,
        words: &mut BTreeMap<String, Vec<ParseIntermediate>>,
        tags: &mut HashSet<Tag>,
        lemma_row: Vec<OpCLid>,
        vangas_words: &mut Vec<String>,
    ) -> Result<(), DictionaryErr> {
        // В начальной форме содержатся общие для леммы граммемы, как одушевленность, часть речи и т.п.
        // Эти теги должны быть вычленены и прокинуты всем словам.
        let inizio_grammemes = self.first_tags()?;

        match &self.variants.as_ref().filter(|v| !v.is_empty()) {
            None => {
                // Если у нас нет других форм, кроме начальной, мы сохраняем просто начальную.
                let word = &self.normal_form.text;

                // todo release 0.2.1 ё.
                let v_word = word.replace('ё', "е");
                vangas_words.push(v_word.clone());

                let form = match first {
                    Lemmatization::Normal => Form::Word(FWord::Normal(lemma_id)),
                    Lemmatization::Inizio => Form::Word(FWord::Inizio(lemma_id)),
                };

                Self::to_word_tags(
                    inizio_grammemes,
                    words,
                    tags,
                    word,
                    &normalization.unwrap_or(word.into()),
                    form,
                    lemma_row,
                )
            }
            Some(forms) => {
                let mut iter = Self::forms(forms.to_vec());
                let normal_form = &self.normal_form.text;

                // todo release 0.2.1 ё.
                let v_word = normal_form.replace('ё', "е");
                vangas_words.push(v_word.clone());

                // Первая форма аналогична начальной, поэтому мы просто совмещаем теги и сохраняем за первой формой FWord.
                {
                    let (first_word, mut first_grammemes) =
                        iter.next().ok_or(DictionaryErr::NoForms(lemma_id))?;
                    first_grammemes.extend(inizio_grammemes.clone());

                    // todo release 0.2.1 ё.
                    let v_word = first_word.replace('ё', "е");
                    vangas_words.push(v_word.clone());

                    let form = match first {
                        Lemmatization::Normal => Form::Word(FWord::Normal(lemma_id)),
                        Lemmatization::Inizio => Form::Word(FWord::Inizio(lemma_id)),
                    };

                    Self::to_word_tags(
                        first_grammemes,
                        words,
                        tags,
                        &first_word,
                        &normalization.clone().unwrap_or(normal_form.clone()),
                        form,
                        lemma_row.clone(),
                    )
                }

                for (text, mut grammemes) in iter {
                    grammemes.extend(inizio_grammemes.clone());
                    let form = Form::Word(FWord::Different(lemma_id));

                    // todo release 0.2.1 ё.
                    let v_word = text.replace('ё', "е");
                    vangas_words.push(v_word.clone());

                    Self::to_word_tags(
                        grammemes,
                        words,
                        tags,
                        &text,
                        &normalization.clone().unwrap_or(normal_form.to_owned()),
                        form,
                        lemma_row.clone(),
                    )
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
        // Само слово
        word: &str,
        // Его нормализованная форма
        normal_form: &str,
        // Форма слова
        form: Form,
        lemma_row: Vec<OpCLid>,
    ) {
        tags.insert(grammemes.clone());
        let parses = words.entry(word.to_lowercase()).or_default();

        parses.push(ParseIntermediate {
            tag: grammemes,
            form,
            normal_form: normal_form.into(),
            opcorp_lemma: lemma_row,
        })
    }
}

pub type LemmaId = u64;
pub type VariationId = u64;

impl Links {
    /// Некоторые части речи/формы зависят от слов, которые не являются морфологическими нормальными формами.
    /// Эти "некоторые" части - вторая степень вложенности к нормальной форме, которую также надо найти.
    ///
    /// Тип 6 - это тип краткого причастия к полному причастию. Но причастие также сводится к инфинитиву. Поэтому краткое -> полное - это второй уровень нормализации.
    /// Тип 22 - это опечатки к правильному правописанию слова. Но последнее может иметь свою нормальную форму, к которой опечатки должны свестись.
    pub(crate) const DOUBLE_FROM: [u64; 2] = [6, 22];

    /// Исключенные связи между леммами.
    /// Исключение связей рассматривалось по Pymorphy и нуждам компании.
    // 11 - не связываем imperfect и perfect.
    // 16, 18 - не связываем сравнительные формы на "-йший" к простому прилагательному.
    // 7, 21, 23, 27 - наследие от Pymorphy + по запросу коллег.
    // 8, 9 было убрано по запросу.
    pub(crate) const EXCLUDED_LINKS: [u64; 9] = [7, 8, 9, 11, 16, 18, 21, 23, 27];

    /// Сбор лемм словаря OpenCorpora по связям между ними.
    /// Ключ - нормализованная форма, значение - все остальные формы, восходящие к нормализованной.
    pub fn collect_lemmas(self) -> HashMap<LemmaId, Vec<VariationId>> {
        let mut link_connotation = HashMap::new();

        let links = self.links.clone();

        'links: for Link {
            type_id,
            lemma_id,
            variant,
        } in self.links
        {
            if Self::EXCLUDED_LINKS.contains(&type_id) {
                continue 'links;
            }

            if Self::DOUBLE_FROM.contains(&type_id) {
                if let Some(real_lemma) = links.iter().find(|link| link.variant == lemma_id) {
                    let variations: &mut Vec<VariationId> =
                        link_connotation.entry(real_lemma.lemma_id).or_default();

                    if !variations.contains(&lemma_id) {
                        variations.push(lemma_id);
                    }

                    if !variations.contains(&variant) {
                        variations.push(variant);
                    }
                } else {
                    let variations: &mut Vec<VariationId> =
                        link_connotation.entry(lemma_id).or_default();

                    if !variations.contains(&variant) {
                        variations.push(variant);
                    }
                }
            } else {
                let variations: &mut Vec<VariationId> =
                    link_connotation.entry(lemma_id).or_default();

                if !variations.contains(&variant) {
                    variations.push(variant);
                }
            }
        }

        link_connotation
    }
}

#[cfg(test)]
pub(crate) mod test {
    use tempfile::tempdir;

    use super::*;
    use crate::{
        gram, grams,
        opencorpora::dictionary::{Gram, NormalForm},
        test_infrastructure::infrastructure::make_dict,
        Method, MorphAnalyzer, ParsedWord,
    };
    use smallvec::SmallVec;
    use test_case::test_case;

    #[test_case(LemmaDict { normal_form: NormalForm { text: "ёж".to_owned(), gram: Some(vec![Gram { v: gram![ParteSpeech::Noun] }]) }, variants: None }, grams![ParteSpeech::Noun].into())]
    fn test_normal_tags(lemma: LemmaDict, tag: Tag) {
        assert_eq!(lemma.first_tags().unwrap(), tag);
    }

    #[test]
    fn test_normalization_small() {
        let dict = DictionaryOpenCorpora::init_from_path("data/test/test_lemma.xml").unwrap();

        let hash = dict.links.collect_lemmas();
        assert_eq!(hash.len(), 2)
    }

    #[test]
    /// Проверка, что у нас не пропадают леммы без связей с другими леммами.
    fn test_no_link_lemma() {
        let tmp_dir = tempdir().unwrap();
        let fst = tmp_dir.path().join("no_link_lemma.fst");

        let dict = make_dict("data/test/senza_grams.xml", fst);
        assert_eq!(dict.lemmas.len(), 3);
    }

    #[test]
    /// Иногда в формах леммы нет никаких дополнительных граммем.
    /// Тест-проверка на то, что такие формы не пропадают, а имеют только граммем начальной формы.
    fn test_no_form() {
        let tmp_dir = tempdir().unwrap();
        let fst = tmp_dir.path().join("no_form.fst");

        let dict = make_dict("data/test/piccolo_dict.xml", fst);
        assert_eq!(dict.word_parses.len(), 4usize);
    }

    #[test]
    fn test_form_flow() {
        let tmp_dir = tempdir().unwrap();
        let fst = tmp_dir.path().join("dict.fst");

        let dict = make_dict("data/test/test_bolshe.xml", fst);
        let anal = MorphAnalyzer::init(dict, tmp_dir).unwrap();

        assert_eq!(
            anal.parse_get("больше", 0).unwrap().unwrap(),
            ParsedWord {
                word: "больше".to_string(),
                tags: SmallVec::from(grams![ParteSpeech::Comparative, Other::Quality]),
                normal_form: "большой".to_string(),
                method: Method::Dictionary
            }
        );
    }

    #[ignore = "Manual collect for representation"]
    #[test]
    fn collect_links() {
        struct Chain {
            from: String,
            to: String,
        }

        let dict = DictionaryOpenCorpora::init_from_path("dict.opcorpora.xml").unwrap();

        let mut stopper: Vec<u64> = Vec::new();

        let mut hash_map: BTreeMap<u64, Vec<Chain>> = BTreeMap::new();
        let iter = dict.lemmata.lemmas.iter();

        for Link {
            type_id,
            lemma_id,
            variant,
        } in dict.links.links
        {
            if stopper.contains(&type_id) {
                continue;
            };

            let from = &iter
                .clone()
                .find(|lemma| lemma.id == lemma_id)
                .unwrap()
                .normal_form
                .text
                .to_owned();
            let to = &iter
                .clone()
                .find(|lemma| lemma.id == variant)
                .unwrap()
                .normal_form
                .text;

            let chains = hash_map.entry(type_id).or_default();
            if chains.len() == 7 {
                stopper.push(type_id);
                continue;
            } else {
                chains.push(Chain {
                    from: from.into(),
                    to: to.into(),
                })
            }
        }

        for (k, v) in hash_map {
            eprintln!("Link type: {k}.");
            for Chain { from, to } in v {
                eprintln!("\tСлово '{to}' будет приводиться к '{from}'");
            }
        }

        panic!()
    }

    #[ignore = "Manual collect for representation"]
    #[test]
    fn collect_links_with_links() {
        let dict = DictionaryOpenCorpora::init_from_path("dict.opcorpora.xml").unwrap();

        let iter = dict.lemmata.lemmas.iter();
        let links = dict.links.collect_lemmas();

        for (k, v) in links.iter().take(1000) {
            let normal = &iter
                .clone()
                .find(|lemma| lemma.id == *k)
                .unwrap()
                .normal_form
                .text;

            let words = v
                .iter()
                .clone()
                .map(|v| {
                    &iter
                        .clone()
                        .find(|lemma| lemma.id == *v)
                        .unwrap()
                        .normal_form
                        .text
                })
                .collect_vec();

            eprintln!(
                "\tК нормальной форме '{normal}' будут приводиться следующие слова '{words:?}'"
            );
        }
    }
}
