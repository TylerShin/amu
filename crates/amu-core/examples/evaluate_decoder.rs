use amu_core::{Input, Mode, Session};
use std::{collections::BTreeMap, time::Instant};

#[path = "../tests/support/mixed_boundaries.rs"]
mod mixed_boundaries;

const FIXTURES: &str = include_str!("../tests/fixtures/decoder.tsv");
const WORD_HINTS: &str = include_str!("../src/english-seed.txt");
const CONSONANTS: &[u8] = b"bcdfghjklmnpqrstvwxyz";
const PREFIXES: &[(&str, &str)] = &[
    ("tkatjd", "삼성"),
    ("tjfwjd", "설정"),
    ("ghkaus", "화면"),
    ("rlsmd", "기능"),
    ("ahepf", "모델"),
    ("dpswls", "엔진"),
];
const CONTEXTS: &[(&str, &[&str])] = &[
    ("neutral", &[]),
    ("after 1 Korean word", &["gksrmf"]),
    ("after 4 Korean words", &["gksrmf"; 4]),
    ("after 1 English word", &["hello"]),
    ("after 4 English words", &["hello"; 4]),
];

fn main() {
    let started = Instant::now();
    std::hint::black_box(Session::new(Mode::Auto).unwrap());
    println!(
        "cold session/model initialization: {:.2}ms",
        started.elapsed().as_secs_f64() * 1000.0
    );
    report_regressions();
    report_short_inputs();
    report_mixed_boundaries();
    report_generated_initialisms(0x5a91_c3d7, "development");
    report_generated_initialisms(0xa720_4e91, "held-out");
    if let Some(path) = std::env::args().nth(1) {
        report_korean_corpus(&path, std::env::args().nth(2).as_deref());
    }
    report_latency();
}

fn report_short_inputs() {
    let mut correct = 0;
    let mut total = 0;
    for line in FIXTURES
        .lines()
        .filter(|line| line.starts_with("short-english\t") || line.starts_with("korean-single\t"))
    {
        let fields: Vec<_> = line.split('\t').collect();
        for (_, context) in CONTEXTS {
            correct += usize::from(decode_after(fields[1], context) == fields[2]);
            total += 1;
        }
    }
    println!("short Latin/Korean fixtures across 5 contexts: {correct}/{total}");
    correct = 0;
    total = 0;
    for first in b'A'..=b'Z' {
        if b"REQTWOP".contains(&first) {
            continue;
        }
        for second in b'A'..=b'Z' {
            let keys = String::from_utf8(vec![first, second]).unwrap();
            for (_, context) in CONTEXTS {
                correct += usize::from(decode_after(&keys, context) == keys);
                total += 1;
            }
        }
    }
    println!(
        "two-letter capitals without a Korean Shift onset across 5 contexts: {correct}/{total}"
    );
}

fn report_mixed_boundaries() {
    let mut cases = Vec::new();
    for prefix in mixed_boundaries::UNREGISTERED_PREFIXES {
        for (keys, text) in mixed_boundaries::KOREAN_SUFFIXES {
            cases.push((format!("{prefix}{keys}"), format!("{prefix}{text}")));
        }
    }
    let korean: Vec<_> = mixed_boundaries::KOREAN_WORDS
        .iter()
        .map(|(keys, text)| (keys.to_string(), text.to_string()))
        .collect();
    let english: Vec<_> = mixed_boundaries::ENGLISH_WORDS
        .iter()
        .map(|text| (text.to_string(), text.to_string()))
        .collect();
    let connected: Vec<_> = mixed_boundaries::CONNECTED_ENGLISH_WORDS
        .iter()
        .map(|text| (text.to_string(), text.to_string()))
        .collect();
    let mut extended = Vec::new();
    for word in mixed_boundaries::CONNECTED_ENGLISH_WORDS {
        for (keys, text) in mixed_boundaries::KOREAN_SUFFIXES {
            extended.push((format!("{word}{keys}"), format!("{word}{text}")));
        }
    }
    for (name, cases) in [
        ("mixed boundaries", cases),
        ("Korean boundary counterexamples", korean),
        ("English boundary counterexamples", english),
        ("unregistered connected English spelling", connected),
        ("Korean after connected English spelling", extended),
    ] {
        let mut correct = 0;
        let mut total = 0;
        let mut misses = Vec::new();
        for (keys, expected) in cases {
            for (context_name, context) in CONTEXTS {
                let actual = decode_after(&keys, context);
                correct += usize::from(actual == expected);
                total += 1;
                if actual != expected && misses.len() < 3 {
                    misses.push(format!("{context_name}: {expected} → {actual}"));
                }
            }
        }
        println!("{name} across 5 contexts: {correct}/{total}");
        for miss in misses {
            println!("  miss: {miss}");
        }
    }
}

fn report_regressions() {
    let mut results = BTreeMap::<&str, (usize, usize)>::new();
    for line in FIXTURES.lines().filter(|line| !line.starts_with('#')) {
        let fields: Vec<_> = line.split('\t').collect();
        let [category, keys, expected, _] = fields.as_slice() else {
            panic!("invalid fixture")
        };
        let actual = decode(keys);
        let counts = results.entry(category).or_default();
        counts.0 += usize::from(actual == *expected);
        counts.1 += 1;
        if actual != *expected {
            println!("regression mismatch: {keys} → {actual}; intended {expected}");
        }
    }
    for (category, (correct, total)) in results {
        println!("regression {category}: {correct}/{total}");
    }
}

// Development challenge, separate from the fixed regression fixtures.
// Some key sequences are genuinely ambiguous with Korean. Report all misses.
fn report_generated_initialisms(mut seed: u64, sample: &str) {
    for uppercase in [false, true] {
        let mut correct = 0;
        let mut total = 0;
        let mut examples = Vec::new();
        for _ in 0..100 {
            seed = next_random(seed);
            let length = 3 + seed as usize % 3;
            let mut letters = String::new();
            for _ in 0..length {
                seed = next_random(seed);
                letters.push(CONSONANTS[seed as usize % CONSONANTS.len()] as char);
            }
            if WORD_HINTS
                .split_ascii_whitespace()
                .any(|word| word == letters)
            {
                continue;
            }
            if uppercase {
                letters.make_ascii_uppercase();
            }
            for (keys, korean) in PREFIXES {
                let actual = decode(&format!("{keys}{letters}"));
                let expected = format!("{korean}{letters}");
                correct += usize::from(actual == expected);
                total += 1;
                if actual != expected && examples.len() < 8 {
                    examples.push(format!("{expected} → {actual}"));
                }
            }
        }
        let case = if uppercase { "uppercase" } else { "lowercase" };
        println!("{sample} generated {case}: {correct}/{total} intended Latin suffixes preserved");
        for example in examples {
            println!("  miss: {example}");
        }
    }
}

fn report_korean_corpus(path: &str, snapshot_path: Option<&str>) {
    let source = std::fs::read_to_string(path).expect("held-out Korean corpus");
    let mut results = BTreeMap::<(usize, &str), (usize, usize, Vec<String>)>::new();
    let mut snapshot = String::new();
    for (line_number, line) in source.lines().enumerate() {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 10 || fields[0].parse::<usize>().is_err() {
            continue;
        }
        let text = fields[1];
        if text.is_empty() || !text.chars().all(|c| ('가'..='힣').contains(&c)) {
            continue;
        }
        let syllables = text.chars().count().min(2);
        let keys = korean_keys(text);
        for (name, context) in CONTEXTS
            .iter()
            .filter(|(name, _)| !name.starts_with("after 1"))
        {
            let actual = decode_after(&keys, context);
            if snapshot_path.is_some() {
                snapshot.push_str(&format!("{line_number}\t{name}\t{text}\t{actual}\n"));
            }
            let (correct, total, examples) = results.entry((syllables, name)).or_default();
            *total += 1;
            *correct += usize::from(actual == text);
            if actual != text && examples.len() < 3 {
                examples.push(format!("{text} → {actual}"));
            }
        }
    }
    if let Some(path) = snapshot_path {
        std::fs::write(path, snapshot).expect("write explicit corpus evaluation snapshot");
    }
    for ((syllables, context), (correct, total, examples)) in results {
        let length = if syllables == 1 {
            "1 syllable"
        } else {
            "2+ syllables"
        };
        println!("held-out Korean FORM tokens ({length}, {context}): {correct}/{total} preserved");
        for example in examples {
            println!("  miss: {example}");
        }
    }
}

fn korean_keys(text: &str) -> String {
    const ONSETS: [&str; 19] = [
        "r", "R", "s", "e", "E", "f", "a", "q", "Q", "t", "T", "d", "w", "W", "c", "z", "x", "v",
        "g",
    ];
    const VOWELS: [&str; 21] = [
        "k", "o", "i", "O", "j", "p", "u", "P", "h", "hk", "ho", "hl", "y", "n", "nj", "np", "nl",
        "b", "m", "ml", "l",
    ];
    const CODAS: [&str; 28] = [
        "", "r", "R", "rt", "s", "sw", "sg", "e", "f", "fr", "fa", "fq", "ft", "fx", "fv", "fg",
        "a", "q", "qt", "t", "T", "d", "w", "c", "z", "x", "v", "g",
    ];
    let mut keys = String::new();
    for character in text.chars() {
        let syllable = character as usize - 0xac00;
        keys.push_str(ONSETS[syllable / 588]);
        keys.push_str(VOWELS[syllable % 588 / 28]);
        keys.push_str(CODAS[syllable % 28]);
    }
    keys
}

fn report_latency() {
    // Warm the shared English statistics once, outside the timed key path.
    decode("hello");
    let typical: Vec<_> = FIXTURES
        .lines()
        .filter(|line| !line.starts_with('#'))
        .map(|line| line.split('\t').nth(1).unwrap().to_owned())
        .collect();
    let capped = vec![format!("{}rksapi", "rk".repeat(29))];
    for (name, words) in [("fixture keys", typical), ("64-key token", capped)] {
        let mut session = Session::new(Mode::Auto).unwrap();
        let mut samples = Vec::new();
        for _ in 0..100 {
            for keys in &words {
                session.reset();
                for key in keys.bytes() {
                    let started = Instant::now();
                    std::hint::black_box(session.process(Input::Letter(key)));
                    samples.push(started.elapsed().as_nanos());
                }
            }
        }
        samples.sort_unstable();
        let percentile = |p: usize| samples[(samples.len() - 1) * p / 100] as f64 / 1000.0;
        println!(
            "release core {name}: n={}, p50={:.1}µs p95={:.1}µs p99={:.1}µs",
            samples.len(),
            percentile(50),
            percentile(95),
            percentile(99)
        );
    }
}

fn decode(keys: &str) -> String {
    decode_after(keys, &[])
}

fn decode_after(keys: &str, context: &[&str]) -> String {
    let mut session = Session::new(Mode::Auto).unwrap();
    for word in context {
        for key in word.bytes() {
            session.process(Input::Letter(key));
        }
        session.process(Input::WordBoundary);
    }
    let mut committed = String::new();
    for key in keys.bytes() {
        committed.push_str(&session.process(Input::Letter(key)).commit);
    }
    committed.push_str(&session.process(Input::Finish).commit);
    committed
}

fn next_random(seed: u64) -> u64 {
    seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1)
}
