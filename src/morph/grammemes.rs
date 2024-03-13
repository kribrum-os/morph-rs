use allocative::Allocative;
use serde::{Deserialize, Serialize};

pub trait ToGrammem {
    fn to_grammem(self) -> Grammem;
}

#[rustfmt::skip]
#[derive(Debug, derive_more::Display, Copy, Clone, Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[display(fmt = "{}", _0.display())]
#[serde(untagged)]
pub enum Grammem {
    /// Часть речи
    ParteSpeech(ParteSpeech),
    /// Одушевленность
    Animacy(Animacy),
    /// Вид: Совершенный (true), несовершенный (false) вид
    Aspect(Aspect),
    /// Падеж
    Case(Case),
    Gender(Gender),
    /// Включенность говорящего в действие
    Involvement(Involvement),
    /// Наклонение: повелительное, изъявительное
    Mood(Mood),
    /// Лицо: единственное, множественное
    Number(Number),
    /// Переходный (true), непереходный (false)
    Trans(Transitivity),
    /// Время
    Tense(Tense),
    /// Залог
    Voice(Voice),
    /// Категория лица
    Person(Person),
    Other(Other),
}

impl Grammem {
    pub fn pos(&self) -> Option<ParteSpeech> {
        match self {
            Grammem::ParteSpeech(p) => Some(*p),
            _ => None,
        }
    }

    pub fn pos_in_tag(vec: &[Self]) -> Option<ParteSpeech> {
        vec.iter().find_map(|t| t.pos())
    }
}

#[rustfmt::skip]
#[derive(Debug, derive_more::Display, Copy, Clone, Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[display(fmt = "{}", _0.display())]
pub enum ParteSpeech {
    #[serde(rename = "NOUN")]
    Noun,
    #[serde(rename = "ADJF")]
    /// Имя прилагательное в полной форме
    AdjectiveFull,
    #[serde(rename = "ADJS")]
    /// Имя прилагательное в краткой форме
    AdjectiveShort,
    #[serde(rename = "COMP")]
    /// Компаратив
    Comparative,
    #[serde(rename = "VERB")]
    /// Глагол, личная форма
    Verb,
    #[serde(rename = "INFN")]
    /// Глагол, инфинитив
    Infinitive,
    #[serde(rename = "PRTF")]
    /// Причастие полное
    ParticipleFull,
    #[serde(rename = "PRTS")]
    /// Причастие краткое
    ParticipleShort,
    #[serde(rename = "GRND")]
    Gerundive,
    #[serde(rename = "NUMR")]
    Number,
    #[serde(rename = "ADVB")]
    /// Наречие
    Adverb,
    #[serde(rename = "NPRO")]
    /// Местоимение-существительное
    NounPronoun,
    #[serde(rename = "PRED")]
    /// Предикатив
    Predicative,
    #[serde(rename = "PREP")]
    /// Предлог
    Preposition,
    #[serde(rename = "CONJ")]
    /// Союз
    Conjunction,
    #[serde(rename = "PRCL")]
    /// Частица
    Particle,
    #[serde(rename = "INTJ")]
    /// Междометие
    Interjection,
}

impl ToGrammem for ParteSpeech {
    fn to_grammem(self) -> Grammem {
        Grammem::ParteSpeech(self)
    }
}

#[rustfmt::skip]
#[derive(Debug, derive_more::Display, Copy, Clone, Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord, Allocative)]
pub enum Form {
    Word(FWord),
    Vanga(FVanga),
}

impl Form {
    pub fn switch_vanga(self) -> Self {
        match self {
            Form::Word(w) => match w {
                FWord::Normal(_) => Form::Vanga(FVanga::Normal),
                FWord::Inizio(_) => Form::Vanga(FVanga::Inizio),
                FWord::Different(_) => Form::Vanga(FVanga::Different),
            },
            v => v,
        }
    }

    pub fn is_normal(&self) -> bool {
        match self {
            Form::Word(w) => matches!(w, FWord::Normal(_)),
            Form::Vanga(v) => matches!(v, FVanga::Normal),
        }
    }

    pub fn is_inizio(&self) -> bool {
        match self {
            Form::Word(w) => matches!(w, FWord::Inizio(_)),
            Form::Vanga(v) => matches!(v, FVanga::Inizio),
        }
    }

    pub fn id(&self) -> Option<u64> {
        match self {
            Form::Word(FWord::Normal(i)) => Some(*i),
            Form::Word(FWord::Inizio(i)) => Some(*i),
            Form::Word(FWord::Different(i)) => Some(*i),
            _ => None,
        }
    }
}

#[rustfmt::skip]
#[derive(Debug, derive_more::Display, Copy, Clone, Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord, Allocative)]
pub enum FWord {
    #[display(fmt = "Normal {_0}")]
    Normal(u64),
    // Начальная форма, но не нормализованная.
    // К примеру, начальная форма деепричастия, у которого нормализованной формой, однако, является глагол.
    #[display(fmt = "Inizio {_0}")]
    Inizio(u64),
    #[display(fmt = "Not normalize {_0}")]
    Different(u64),
}

#[rustfmt::skip]
#[derive(Debug, derive_more::Display, Copy, Clone, Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord, Allocative)]
pub enum FVanga {
    Normal,
    // Начальная форма, но не нормализованная.
    // К примеру, начальная форма деепричастия, у которого нормализованной формой, однако, является глагол.
    Inizio,
    Different,
}

#[rustfmt::skip]
#[derive(Debug, derive_more::Display, Copy, Clone, Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[display(fmt = "{}", _0.display())]
#[serde(rename = "$value")]
pub enum Person {
    #[serde(rename = "1per")]
    First,
    #[serde(rename = "2per")]
    Second,
    #[serde(rename = "3per")]
    Third,
    #[serde(rename = "Impe")]
    Impersonal,
    #[serde(rename = "Impx")]
    PossibleImpersonal,
}

impl ToGrammem for Person {
    fn to_grammem(self) -> Grammem {
        Grammem::Person(self)
    }
}

#[rustfmt::skip]
#[derive(Debug, derive_more::Display, Copy, Clone, Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[display(fmt = "{}", _0.display())]
#[serde(rename = "$value")]
pub enum Animacy {
    #[serde(rename = "anim")]
    Animate,
    #[serde(rename = "inan")]
    Inanimate,
    /// Может использоваться как одуш. / неодуш. 
    #[serde(rename = "Inmx")]
    Both,
}

impl ToGrammem for Animacy {
    fn to_grammem(self) -> Grammem {
        Grammem::Animacy(self)
    }
}

#[rustfmt::skip]
#[derive(Debug, derive_more::Display, Copy, Clone, Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[display(fmt = "{}", _0.display())]
#[serde(rename = "$value")]
pub enum Aspect {
    #[serde(rename = "perf")]
    /// Совершенный
    Perfetto,
    #[serde(rename = "impf")]
    /// Несовершенный
    Imperfetto,
}

impl ToGrammem for Aspect {
    fn to_grammem(self) -> Grammem {
        Grammem::Aspect(self)
    }
}

#[rustfmt::skip]
#[derive(Debug, derive_more::Display, Copy, Clone, Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[display(fmt = "{}", _0.display())]
#[serde(rename = "$value")]
pub enum Number {
    #[serde(rename = "sing")]
    Singular,
    #[serde(rename = "plur")]
    Plural,
    #[serde(rename = "Sgtm")]
    /// Всегда используется в единственном числе
    SingulariaTantum,
    /// Всегда используется в множественном числе
    #[serde(rename = "Pltm")]
    PluraliaTantum,
}

impl ToGrammem for Number {
    fn to_grammem(self) -> Grammem {
        Grammem::Number(self)
    }
}

impl Number {
    pub fn to_default(self) -> Self {
        match self {
            Number::Plural => Number::Singular,
            otherwise => otherwise,
        }
    }
}

#[rustfmt::skip]
#[derive(Debug, derive_more::Display, Copy, Clone, Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[display(fmt = "{}", _0.display())]
#[serde(rename = "$value")]
pub enum Transitivity {
    #[serde(rename = "tran")]
    /// Переходный
    Transitive,
    #[serde(rename = "intr")]
    /// Непереходный
    Intransitive,
}

impl ToGrammem for Transitivity {
    fn to_grammem(self) -> Grammem {
        Grammem::Trans(self)
    }
}

#[rustfmt::skip]
#[derive(Debug, derive_more::Display, Copy, Clone, Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[display(fmt = "{}", _0.display())]
#[serde(rename = "$value")]
pub enum Tense {
    #[serde(rename = "past")]
    Past,
    #[serde(rename = "pres")]
    Present,
    #[serde(rename = "futr")]
    Future,
}

impl ToGrammem for Tense {
    fn to_grammem(self) -> Grammem {
        Grammem::Tense(self)
    }
}

#[rustfmt::skip]
#[derive(Debug, derive_more::Display, Default, Copy, Clone, Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[display(fmt = "{}", _0.display())]
#[serde(rename = "$value")]
pub enum Case {
    // Неизменяемое
    #[serde(rename = "Fixd")]
    Fixed,
    // Именительный
    #[default]
    #[serde(rename = "nomn")]
    Nominativus,
    // Родительный
    #[serde(rename = "gent")]
    // Следующее приведение используется в Pymorphy2.
    #[serde(alias = "gen1")]
    Genetivus,
    // Дательный
    #[serde(rename = "datv")]
    Dativus,
    // Винительный
    #[serde(rename = "accs")]
    // Следующее приведение используется в Pymorphy2.
    #[serde(alias = "acc1")]
    Accusativus,
    // Творительный
    #[serde(rename = "ablt")]
    Ablativus,
    // Предложный
    #[serde(rename = "loct")]
    // Следующее приведение используется в Pymorphy2.
    #[serde(alias = "loc1")]
    Locativus,
    // Звательный
    #[serde(rename = "voct")]
    Vocativus,

    #[serde(rename = "gen2")]
    Gen2,
    #[serde(rename = "acc2")]
    Acc2,
    #[serde(rename = "loc2")]
    Loc2,
}

impl ToGrammem for Case {
    fn to_grammem(self) -> Grammem {
        Grammem::Case(self)
    }
}

#[rustfmt::skip]
#[derive(Debug, derive_more::Display, Copy, Clone, Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[display(fmt = "{}", _0.display())]
#[serde(rename = "$value")]
#[allow(clippy::enum_variant_names)]
pub enum Gender {
    #[serde(rename = "masc")]
    Masculine,
    #[serde(rename = "femn")]
    Feminine,
    #[serde(rename = "neut")]
    Neutral,
    /// Общий род (м/ж),
    #[serde(rename = "ms-f")]
    Common,
    /// Колебание по роду (м/ж/с): кофе, вольво
    #[serde(rename = "Ms-f")]
    CommonWavering,
    /// Род / род не выражен
    #[serde(rename = "GNdr")]
    GenderNeutral,

}

impl ToGrammem for Gender {
    fn to_grammem(self) -> Grammem {
        Grammem::Gender(self)
    }
}

#[rustfmt::skip]
#[derive(Debug, derive_more::Display, Copy, Clone, Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[display(fmt = "{}", _0.display())]
#[serde(rename = "$value")]
pub enum Mood {
    #[serde(rename = "indc")]
    // Изъяснительное
    Indicativo,
    #[serde(rename = "impr")]
    // Повелительное
    Imperativo,
}

impl ToGrammem for Mood {
    fn to_grammem(self) -> Grammem {
        Grammem::Mood(self)
    }
}

#[rustfmt::skip]
#[derive(Debug, derive_more::Display, Copy, Clone, Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[display(fmt = "{}", _0.display())]
#[serde(rename = "$value")]
pub enum Voice {
    #[serde(rename = "actv")]
    // Действительный
    Active,
    #[serde(rename = "pssv")]
    // Страдательный
    Passive,
}

impl ToGrammem for Voice {
    fn to_grammem(self) -> Grammem {
        Grammem::Voice(self)
    }
}

#[rustfmt::skip]
#[derive(Debug, derive_more::Display, Copy, Clone, Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[display(fmt = "{}", _0.display())]
#[serde(rename = "$value")]
pub enum Involvement {
    #[serde(rename = "incl")]
    /// Говорящий включен в действие
    Incluso,
    #[serde(rename = "excl")]
    /// Говорящий не включен в действие
    Excluso,
}

impl ToGrammem for Involvement {
    fn to_grammem(self) -> Grammem {
        Grammem::Involvement(self)
    }
}

#[rustfmt::skip]
#[derive(Debug, derive_more::Display, Copy, Clone, Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[display(fmt = "{}", _0.display())]
#[serde(rename = "$value")]
#[allow(clippy::enum_variant_names)]
pub enum Other {
    /// Аббревиатура
    #[serde(rename = "Abbr")]
    Abbreviation,
    #[serde(rename = "Name")]
    Name,
    #[serde(rename = "Surn")]
    Surname,
    #[serde(rename = "Patr")]
    Patronymic,
    #[serde(rename = "Geox")]
    Geography,
    #[serde(rename = "Orgn")]
    Organization,
    #[serde(rename = "Trad")]
    Trademark,

    /// Возможно субстантивация
    #[serde(rename = "Subx")]
    PossibleSubstantive,
    /// Превосходная степень
    #[serde(rename = "Supr")]
    Superior,
    /// Качественное
    #[serde(rename = "Qual")]
    Quality,
    /// Местоименное
    #[serde(rename = "Apro")]
    Pronominal,
    /// Порядковое
    #[serde(rename = "Anum")]
    Ordinal,
    /// Притяжательное
    #[serde(rename = "Poss")]
    Possessive,
    /// Вопросительное
    #[serde(rename = "Ques")]
    Questionable,
    /// Указательное
    #[serde(rename = "Dmns")]
    Demonstrative,
    /// Анафорическое (местоимение)
    #[serde(rename = "Anph")]
    Anaphoric,

    /// Сравнительная степень на по-
    #[serde(rename = "Cmp2")]
    Comparative,
    /// Форма на еею
    #[serde(rename = "V-ey")]
    FormEY,
    /// Форма на еою
    #[serde(rename = "V-oy")]
    FormOY,
    /// Форма на -ей
    #[serde(rename = "V-ej")]
    FormEJ,
    /// Форма на -ье
    #[serde(rename = "V-be")]
    FormBE,
    /// Форма на -енен
    #[serde(rename = "V-en")]
    FormENEN,
    /// Форма на -и- (веселие, твердостию); отчество с -ие
    #[serde(rename = "V-ie")]
    FormIE,
    /// Форма на -ьи
    #[serde(rename = "V-bi")]
    FormBI,
    /// деепричастие на -ши
    #[serde(rename = "V-sh")]
    ParticipleSH,    

    /// Многократный
    #[serde(rename = "Mult")]
    Multiple,
    /// Возвратный
    #[serde(rename = "Refl")]
    Reflessivo,
    /// Разговорное
    #[serde(rename = "Infr")]
    Spoken,
    /// жаргонное
    #[serde(rename = "Slng")]
    Slang,
    /// Устаревшее
    #[serde(rename = "Arch")]
    Archaic,
    /// Литературный вариант
    #[serde(rename = "Litr")]
    Literary,
    /// Опечатка
    #[serde(rename = "Erro")]
    Error,
    /// Искажение
    #[serde(rename = "Dist")]
    Distortion,
    /// Вводное слово
    #[serde(rename = "Prnt")]
    Parenthesis,
    /// деепричастие от глагола несовершенного вида
    #[serde(rename = "Fimp")]
    ImperfectiveParticiple,
    /// может выступать в роли предикатива
    #[serde(rename = "Prdx")]
    PossiblePredicative,
    /// счётная форма
    #[serde(rename = "Coun")]
    Countable,
    /// Собирательное числительное
    #[serde(rename = "Coll")]
    Collection,
    /// Форма после предлога
    #[serde(rename = "Af-p")]
    AfterPreposition,
    /// Вариант предлога ( со, подо, ...)
    #[serde(rename = "Vpre")]
    PrepositionVariant,
    /// Инициал
    #[serde(rename = "Init")]
    Initial,
    /// Может выступать в роли прилагательного
    #[serde(rename = "Adjx")]
    PossibleAdjective,    
    /// Гипотетическая форма слова (победю, асфальтовее)
    #[serde(rename = "Hypo")]
    Hypothetical,
    #[serde(other)]
    Other, 
}

impl ToGrammem for Other {
    fn to_grammem(self) -> Grammem {
        Grammem::Other(self)
    }
}
