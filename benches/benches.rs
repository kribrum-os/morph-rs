use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use mimalloc::MiMalloc;
use morph_rs::MorphAnalyzer;
use pprof::criterion::{Output, PProfProfiler};
use pyo3::PyResult;
use std::io::Read;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

/// Основная функция инициализации словаря, запускается отдельно.
fn init_benchmark(c: &mut Criterion) {
    let dict_path = "dict.opcorpora.xml";

    // Задаем Throughput в виде самого словаря для подсчета данных/секунду.
    let bytes = std::fs::File::open(dict_path)
        .expect("Open dictionary file")
        .bytes()
        .count() as u64;

    let mut group = c.benchmark_group("mops init");
    group.sample_size(10);
    group.throughput(criterion::Throughput::Bytes(bytes));

    group.bench_function(BenchmarkId::new("init", 0), |b| {
        b.iter(|| {
            black_box(MorphAnalyzer::create(
                dict_path.into(),
                "benches/result/".into(),
                morph_rs::Language::Russian,
            ))
        })
    });
}

/// Основная функция, которая запускает бенчмарки по парсингу, нормализации по словарным словам.
fn benchmark(c: &mut Criterion) {
    let dict_path = "dict.opcorpora.xml";

    let mops = MorphAnalyzer::create(
        dict_path.into(),
        "benches/result/".into(),
        morph_rs::Language::Russian,
    )
    .expect("Mops creation");
    let mops = MorphAnalyzer::init(mops).expect("Mops initialization");

    // Все уникальные слова из Войны и мир
    let binding = std::fs::read_to_string("benches/data/words.txt").expect("Read text file");
    let words = binding.lines();

    // Задаем Throughput для подсчета данных/секунду.
    let bytes = std::fs::File::open("benches/data/words.txt")
        .expect("Open text file")
        .bytes()
        .count() as u64;

    let mut group = c.benchmark_group("Rust 0.1.0. Word&Peace");
    group.throughput(criterion::Throughput::Bytes(bytes));

    group.bench_with_input(
        BenchmarkId::new("parse", 0),
        &(&mops, words.clone()),
        |b, (mops, words)| {
            b.iter(|| {
                for word in words.clone() {
                    // Release 0.1.0: игнорируем ошибку о том, что слова нет в словаре.
                    // Работаем только со словарными словами.
                    let _ = mops.parse(word);
                }
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("normalize", 1),
        &(&mops, words.clone()),
        |b, (mops, words)| {
            b.iter(|| {
                for word in words.clone() {
                    // Release 0.1.0: игнорируем ошибку о том, что слова нет в словаре.
                    // Работаем только со словарными словами.
                    let _ = mops.normalize(word);
                }
            })
        },
    );
}

/// Бенчмарк на работу Pymorphy2(3) со всеми словаря
fn python_bench(c: &mut Criterion) {
    let test_data = std::fs::read_to_string("benches/data/words.txt")
        .expect("Open text file")
        .lines()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    let bytes = test_data.iter().map(|s| s.len()).sum::<usize>();

    let mut group = c.benchmark_group("Pymorphy3. Word&Peace");
    group.throughput(criterion::Throughput::Bytes(bytes as u64));

    pyo3::Python::with_gil::<_, PyResult<()>>(|py| {
        let pymorophy = py.import("pymorphy3")?;
        let morph = pymorophy.getattr("MorphAnalyzer")?.call0()?;

        group.bench_function("parse", |b| {
            b.iter(|| {
                for word in &test_data {
                    let _meta = morph
                        .call_method("parse", (word,), None)
                        .unwrap()
                        .to_string();
                }
            })
        });

        group.bench_function("normalize", |b| {
            b.iter(|| {
                for word in &test_data {
                    let word = morph
                        .call_method("parse", (word,), None)
                        .unwrap()
                        .get_item(0)
                        .unwrap();
                    let _meta = word.getattr("normal_form").unwrap().to_string();
                }
            })
        });

        Ok(())
    })
    .unwrap();
}

criterion_group!(python, python_bench);

criterion_group!(
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = benchmark
);

criterion_group!(init, init_benchmark);

criterion_main!(benches, python, init);
