use clap::{Parser, Subcommand};
use mimalloc::MiMalloc;
use morph_rs::{grams, morph::grammemes::*, Language, MorphAnalyzer};
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
    Declension {
        word: String,
    },
    DeclensionGeography {
        word: String,
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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let Args {
        dictionary,
        db,
        language,
        command,
        init,
    } = Args::parse();

    let start = std::time::Instant::now();

    let anal = match init {
        true => {
            let anal = MorphAnalyzer::create(dictionary, db.clone(), language)?;
            debug!("Инициализация словаря: {:?}", start.elapsed());
            MorphAnalyzer::init(anal, db)?
        }
        false => {
            let anal = MorphAnalyzer::open(db)?;
            debug!("Словарь открывается за: {:?}", start.elapsed());
            anal
        }
    };

    debug!(
        "Весит: {} Мбайт",
        (allocative::size_of_unique_allocated_data(&anal) + anal.fst.len())
            .div(1024)
            .div(1024),
    );

    let new_start = std::time::Instant::now();

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
        Commands::Inflect { word } => println!("{}", anal.inflect_inizio(&word)?.unwrap()),
        Commands::Declension { word } => {
            let parses = anal.declension(&word).unwrap();
            debug!("Declension variations {}", parses.len());

            for (i, parse) in parses.iter().enumerate() {
                println!("For {i} parse:\n{parse}");
            }

            println!("{:?}", new_start.elapsed());
        }
        Commands::DeclensionGeography { word } => {
            let parses = anal
                .parse_grammemes(&word, grams!(Other::Geography))
                .unwrap()
                .unwrap();
            let declension = anal.declension_parsed(&parses).unwrap().unwrap();

            println!("{declension}");

            println!("{:?}", new_start.elapsed());
        }
    };

    Ok(())
}
