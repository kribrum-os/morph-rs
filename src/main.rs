use clap::{Parser, Subcommand};
use itertools::Itertools;
use mimalloc::MiMalloc;
use morph_rs::{Language, MorphAnalyzer, SMALLLEMMA, SMALLTAG, SMALLVANGA};
use std::{ops::Div, path::PathBuf};
use tracing::debug;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(Debug, Subcommand)]
enum Commands {
    Parse {
        word: String,
    },
    ParseGet {
        word: String,
        #[clap(short)]
        index: usize,
    },
    ParseTag {
        word: String,
    },
    Normalize {
        word: String,
    },
    NormalizeGet {
        word: String,
        #[clap(short)]
        index: usize,
    },
    NormalizeTag {
        word: String,
    },
    Inflect {
        word: String,
    },
    Test,
    TestSentence {
        #[clap(value_delimiter = ' ')]
        sentence: Vec<String>,
    },
}

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long = "dict", default_value = "dict.opcorpora.xml")]
    dictionary: PathBuf,

    /// Куда идет fst.
    #[clap(long, default_value = "data/result/")]
    db: PathBuf,

    #[clap(short, default_value = "russian")]
    language: Language,

    #[command(subcommand)]
    command: Commands,

    #[clap(short, long, default_value_t = false)]
    /// Требуется ли инициализация словаря или только открыть его.
    init: bool,

    #[clap(short, long)]
    test_peace: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let Args {
        dictionary,
        db,
        language,
        command,
        init,
        test_peace,
    } = Args::parse();

    let start = std::time::Instant::now();

    let anal = match init {
        true => {
            let dict = MorphAnalyzer::create(dictionary, db, language)?;
            MorphAnalyzer::init(dict)?
        }
        false => MorphAnalyzer::open(db)?,
    };

    debug!("Инициализация словаря: {:?}", start.elapsed());

    debug!(
        "Весит: {}",
        (allocative::size_of_unique_allocated_data(&anal)
            + anal.fst.len()
            + anal.tags.len() * SMALLTAG
            + anal.lemmas.len() * SMALLLEMMA
            + anal
                .paradigms
                .iter()
                .map(|v| v.postfix.len() * SMALLVANGA)
                .collect_vec()
                .len())
        .div(1_000_000)
    );

    match command {
        Commands::Parse { word } => println!("{}", anal.parse(&word)?),
        Commands::ParseGet { word, index } => {
            println!("{:?}", anal.parse_get(&word, index)?)
        }
        Commands::ParseTag { word } => println!("{:?}", anal.parse_get(&word, 0)?.unwrap().tag()),
        Commands::Normalize { word } => println!("{}", anal.normalize(&word)?),
        Commands::NormalizeGet { word, index } => {
            println!("{:?}", anal.normalize_get(&word, index)?)
        }
        Commands::NormalizeTag { word } => {
            println!("{:?}", anal.normalize_get(&word, 0)?.unwrap().tag())
        }
        Commands::Inflect { word } => println!("{}", anal.inflect(&word)?),
        Commands::Test => {
            if test_peace {
                let binding = std::fs::read_to_string("benches/data/words.txt").unwrap();
                let words = binding.lines().collect_vec();
                let words_count = words.clone().len();

                println!("Слов: {words_count}");

                let parse = std::time::Instant::now();

                for word in words.clone() {
                    let _ = anal.parse(word);
                }

                println!("Парсинг: {:?}", parse.elapsed());
                println!(
                    "Парсинг/штука: {:?}",
                    parse.elapsed().div_f32(words_count as f32)
                );

                let normalize = std::time::Instant::now();

                for word in words {
                    let _ = anal.parse(word);
                }

                println!("Нормализация: {:?}", normalize.elapsed());
                println!(
                    "Нормализация/штука: {:?}",
                    normalize.elapsed().div_f32(words_count as f32)
                );
            } else {
                test(&anal)
            }
        }
        Commands::TestSentence { sentence } => {
            for word in sentence {
                println!("{}", anal.parse(&word)?)
            }
        }
    };

    Ok(())
}

fn test(anal: &MorphAnalyzer) {
    for word in WORDS_FOR_NER {
        let word = word.to_lowercase();
        if !anal.is_known(&word) {
            eprintln!("{word}")
        } else {
            eprintln!(
                "Normal form: {}",
                anal.normalize_get(&word, 0).unwrap().unwrap().word()
            )
        }
    }
}

const WORDS_FOR_NER: [&str; 36] = [
    "время",
    "июле",
    "Анне",
    "Павловне",
    "Шерер",
    "Марии",
    "Федоровны",
    "Василия",
    "Инженерной",
    "улице",
    "Алтуфьевском",
    "шоссе",
    "республике",
    "Карелии",
    "телефон",
    "улице",
    "Маршала",
    "Мерецкова",
    "улицей",
    "Крузенштерна",
    "руб",
    "рубля",
    "рублей",
    "рублями",
    "января",
    "феврале",
    "марте",
    "апрелем",
    "маем",
    "июня",
    "июля",
    "августа",
    "сентября",
    "октябрь",
    "ноябрь",
    "декабрь",
];
