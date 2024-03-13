use itertools::Itertools;
use serde::{Deserialize, Serialize};
use smallstr::SmallString;
use smallvec::SmallVec;
use std::collections::HashMap;

use crate::{
    analyzer::{dictionary::LemmaDict, Tag, Tags, Vanga, VangaItem},
    errors::DictionaryErr,
    morph::grammemes::{FVanga, Form},
    SMALLVANGA,
};

use super::{grammemes::Grammem, UNPRODUCTIVE};

/// `Vanga` - предсказание по части речи на основе суффикса.
/// Однако нам необходимо предварительно собрать элементы, чтобы объединить их в целое.
/// Key является набор постфиксов с тегами, value - популярность промежуточной Vanga, которую мы прибавляем каждый раз.
pub type VangaIntermediate = HashMap<Vec<VangaItemIntermediate>, u64>;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Serialize, Deserialize)]
/// Промежуточная единица для образования `VangaIntermediate`.
pub struct VangaItemIntermediate {
    pub(crate) postfix: SmallString<[u8; SMALLVANGA]>,
    pub(crate) form: Form,
    pub(crate) tag: Vec<Tag>,
}

impl Vanga {
    /// Преобразование предопределенных `VangaIntermediate` в конечную `Vanga`
    /// с `VangaItem` и `TagsID` для тегов.
    pub fn parse_vangas(
        stems: VangaIntermediate,
        all_tags: &Tags,
    ) -> Result<Vec<Self>, DictionaryErr> {
        // Количество Ванг у нас может быть не больше собранных промежуточных предсказаний.
        let mut vangas = Vec::with_capacity(stems.len());

        for (inter_items, popularity) in stems {
            // По аналогии с Pymorphy2 убираем непродуктивные парадигмы, т.е. встреченные менее трех раз.
            if popularity < 3 {
                continue;
            }

            let mut items = Vec::new();

            // Граммемы внутри tag уже отсортированы
            for VangaItemIntermediate { postfix, form, tag } in inter_items {
                let mut tags = Vec::with_capacity(tag.capacity());

                for tag in tag {
                    tags.push(
                        all_tags
                            .binary_search(&tag)
                            .map_err(|_| DictionaryErr::BinaryTagVanga(tag.clone()))?,
                    )
                }

                let item = VangaItem {
                    postfix,
                    form,
                    tag: tags,
                };

                items.push(item);
            }

            vangas.push(Vanga {
                popularity,
                postfix: items,
            });
        }

        vangas.sort_by(|a, b| b.popularity.cmp(&a.popularity));
        Ok(vangas)
    }
}

impl VangaItemIntermediate {
    /// Отсекает общий корень и сохраняет постфикс с формой и граммемами слова.
    pub fn parse_vanga_item(
        stem: &str,
        to_vanga_item: &str,
        tag: Tag,
        form: Form,
    ) -> Result<Option<Self>, DictionaryErr> {
        let mut tag = tag
            .into_iter()
            .filter(|grammem| !matches!(grammem, Grammem::Other(_)))
            .collect_vec();
        tag.sort();

        // TODO Ё. release 0.?.?
        let word = to_vanga_item.replace('ё', "е");

        let (_, postfix) = word
            .split_once(stem)
            .ok_or_else(|| DictionaryErr::Stem(stem.to_owned()))?;

        // По аналогии с Pymorphy2 убираем постфиксы более 5 символов.
        let count = postfix.chars().count();
        if count > 0 && count <= 5 {
            Ok(Some(Self {
                postfix: SmallString::from(postfix),
                form,
                tag: vec![SmallVec::from(tag)],
            }))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug)]
pub struct VangaVariant {
    // Мы собираем Ванга-варианты, чтобы в дальнейшем выделить стеммы.
    pub(crate) word: String,
    pub(crate) form: FVanga,
    pub(crate) tag: Tag,
}

#[derive(Debug, Default)]
pub struct LemmaVanga {
    pub(crate) variants: Vec<VangaVariant>,
}

impl LemmaVanga {
    /// Создание `LemmaVanga` по нормализованной форме слова.
    pub fn push_normal(normal: &LemmaDict) -> Result<Self, DictionaryErr> {
        let LemmaDict {
            normal_form,
            variants,
        } = normal;

        let normal_grammemes = LemmaDict::first_tags(normal)?;

        let mut vangas: Vec<VangaVariant> =
            Vec::with_capacity(1 + variants.as_ref().map(|v| v.len()).unwrap_or(0));

        match variants.as_ref().filter(|v| !v.is_empty()) {
            // Если у нас нет других форм, кроме начальной, мы сохраняем просто начальную.
            None => vangas.push(VangaVariant {
                form: FVanga::Normal,
                word: normal_form.text.clone(),
                tag: normal_grammemes,
            }),
            Some(variants) => {
                let mut iter = Self::forms(variants.to_owned().to_owned());

                // Первая форма аналогична начальной, поэтому мы просто совмещаем теги и сохраняем за первой формой FVanga.
                if let Some((first_word, mut first_grammemes)) = iter.next() {
                    first_grammemes.extend(normal_grammemes.clone());

                    vangas.push(VangaVariant {
                        form: FVanga::Normal,
                        word: first_word,
                        tag: first_grammemes,
                    });
                } else {
                    return Err(DictionaryErr::NoFormsVanga(normal_form.text.to_owned()));
                }

                for (word, mut diff_grammemes) in iter {
                    diff_grammemes.extend(normal_grammemes.clone());

                    vangas.push(VangaVariant {
                        form: FVanga::Different,
                        word,
                        tag: diff_grammemes,
                    });
                }
            }
        }

        Ok(Self { variants: vangas })
    }

    /// Если имеются другие формы, не только нормализованная,
    /// необходимо добавить их в общую `LemmaVanga`.
    pub fn update_form(&mut self, another: LemmaDict) -> Result<(), DictionaryErr> {
        let inflect_grammemes = LemmaDict::first_tags(&another)?;

        match another.variants.filter(|v| !v.is_empty()) {
            // Если у нас нет других форм, кроме начальной, мы сохраняем просто начальную.
            None => {
                self.variants.push(VangaVariant {
                    form: FVanga::Inizio,
                    word: another.normal_form.text,
                    tag: inflect_grammemes,
                });
            }
            Some(vars) => {
                let mut iter = Self::forms(vars);

                // Первая форма аналогична начальной, поэтому мы просто совмещаем теги и сохраняем за первой формой FVanga.
                if let Some((first_word, mut first_grammemes)) = iter.next() {
                    first_grammemes.extend(inflect_grammemes.clone());

                    self.variants.push(VangaVariant {
                        form: FVanga::Inizio,
                        word: first_word,
                        tag: first_grammemes,
                    });
                } else {
                    return Err(DictionaryErr::NoFormsVanga(another.normal_form.text));
                }

                for (word, mut diff_grammemes) in iter {
                    diff_grammemes.extend(inflect_grammemes.clone());

                    self.variants.push(VangaVariant {
                        form: FVanga::Different,
                        word,
                        tag: diff_grammemes,
                    });
                }
            }
        }

        Ok(())
    }

    /// Предварительный сбор постфиксов (`Vanga`) с наборами тегов.
    ///
    /// У `Vanga`-и может быть не один набор тегов.
    /// Например, `cтали`, окончание `и` - сущ. в род.пад. **и** гл. мн.ч. в прош.вр.
    pub(crate) fn collect_vangas(
        &self,
        stems: &mut VangaIntermediate,
        words: Vec<String>,
    ) -> Result<(), DictionaryErr> {
        let mut items = Vec::new();
        let stem = longest_common_substring(
            words,
            // self.all_word_forms()
        )
        .to_lowercase();

        if stem.chars().count() < 3 {
            // По аналогии с Pymorphy2 мы не собираем Ванги, если оставшаяся основа меньше трех букв.
        } else {
            // Мы можем определять Ванги только у словарных слов.
            if !self.first_tags()?.iter().any(|g| UNPRODUCTIVE.contains(g)) {
                for VangaVariant { word, form, tag } in
                    self.variants.iter().filter(|vv| !vv.tag.is_empty())
                {
                    let vanga_item = VangaItemIntermediate::parse_vanga_item(
                        &stem,
                        word,
                        tag.to_owned(),
                        Form::Vanga(form.to_owned()),
                    )?;
                    if let Some(vanga_item) = vanga_item {
                        items.push(vanga_item);
                    }
                }
            }
        }

        if !items.is_empty() {
            let popularity = stems.entry(items).or_insert(0);
            *popularity += 1u64;
        }

        Ok(())
    }
}

/// Нахождение максимально длинной основы слова.
pub fn longest_common_substring(data: Vec<String>) -> String {
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
mod test {
    use itertools::Itertools;
    use test_case::test_case;

    use super::longest_common_substring;

    #[test_case(vec!["apricot", "rice", "cricket"] => "ric")]
    #[test_case(vec!["apricot", "banana"] => "a")]
    #[test_case(vec!["foo", "bar", "baz"] => "")]
    #[test_case(vec!["еж", "ежа", "ежу", "ежом"] => "еж")]
    #[test_case(vec!["ежистее", "ежистее", "ежистей", "поежистее", "поежистей"] => "ежисте")]
    fn test_longest_substing(slice: Vec<&str>) -> String {
        let slice = slice.into_iter().map(|s| s.to_string()).collect_vec();
        longest_common_substring(slice)
    }
}
