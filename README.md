# MOrPh-rS

Мопс — морфологический анализатор для русского языка.

- [MOrPh-rS](#morph-rs)
  - [Главное](#главное)
  - [Примеры использования](#примеры-использования)
    - [Создание словаря](#создание-словаря)
    - [Парсинг](#парсинг)
    - [Нормализация](#нормализация)
    - [Склонение](#склонение-слова-в-нужную-форму)
    - [Все формы](#склонениеспряжение-слова-во-все-формы)
  - [Производительность](#производительность)
  - [План развития](#план-развития)
  - [Лицензия](#лицензия)
  - [Благодарности](#благодарности)

## Главное

* Приведение к начальной форме слова.
* Грамматическая характеристика слова: получение грамматической информации о слове.
* Работа с OpenCorpora-совместимыми словарями.
* ✨Производительность✨: скорость разбора в десятки раз превышает PyMorphy2.

## Примеры использования

### Создание словаря

Инициализация морфологического анализатора требует словарь OpenCorpora, представленный [на сайте](https://opencorpora.org/dict.php),
выходной каталог, где будут сохранены бинарные данные, и указание языка (на данный момент имеется только русский язык).

```rust
let dict = MorphAnalyzer::create(dictionary, db, language).unwrap();
let morph = MorphAnalyzer::init(dict).unwrap();
```

### Парсинг

```rust
let morph = MorphAnalyzer::open(dict_path).unwrap();

let stali = morph.parse("стали").unwrap();
println!("{stali}");
```

### Нормализация

```rust
let morph = MorphAnalyzer::open(dict_path).unwrap();

let stali = morph.normalize("стали").unwrap();
println!("{stali}");
```

### Склонение слова в нужную форму.

```rust
let morph = MorphAnalyzer::open(dict_path).unwrap();

let stali = morph.inflect_forms("стали", grams![Gender::Feminine]).unwrap();
println!("{stali:?}");
```


```rust
let morph = MorphAnalyzer::open(dict_path).unwrap();

let stali = morph.parse("стали").unwrap().0[5]; // индекс соответствует глаголу "стать"
let stali = morph.inflect_parsed(stali, grams![Gender::Feminine]).unwrap();
println!("{stali:?}");
```

### Склонение/спряжение слова во все формы.

Возможность привести слово ко всем формам, считая связи между леммами.
Например, стать -> стал, стала, стали, ставший, ставшая, ставшие и т.д.

Функция затратная по производительности. Если есть необходимый набор слов, который нужно будет искать во всех формах в тексте, лучше сделать вызов функции в начале работы приложения.

```rust
let morph = MorphAnalyzer::open(dict_path).unwrap();

let stali = morph.declension("стали").unwrap();
println!("{stali:?}");
```


```rust
let morph = MorphAnalyzer::open(dict_path).unwrap();

let stali = morph.parse("стали").unwrap().0[5]; // индекс соответствует глаголу "стать"
let stali = morph.declension_parsed(stali).unwrap();
println!("{stali:?}");
```

## Производительность

Результат нагрузочного тестирования может быть найден в [benchmarks.md](./benches/benchmarks.md).
Там же находятся результаты сравнительного тестирования с `PyMorphy2`.

## План развития

- [ ] Предсказание грамматических характеристик несловарного слова по постфиксу.
- [ ] Работа со словами с дефисом.
- [ ] Склонение несловарных слов.

## Лицензия

Данный код распространяется под лицензией [Kribrum-NC](./license.md), которая основана на `Apache License Version 2.0`.

## Благодарности

* Руководству [Крибрум](kribrum.ru), которое позволило вывести эту работу в OpenSource.
* Разработчикам [PyMorhpy2](https://github.com/pymorphy2/pymorphy2) за создание источника вдохновения при разработке Мопса.
* Создателям [OpenCorpora](https://opencorpora.org/) за формирование словаря, который Мопс использует по умолчанию.
* Руководителю группы разработки [Nikita Patsakula](https://github.com/npatsakula) за консультации и активное ревью. 
