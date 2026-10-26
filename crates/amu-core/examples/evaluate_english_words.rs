use amu_core::{Input, Mode, Session};
use std::collections::BTreeSet;

const WORD_HINTS: &str = include_str!("../src/english-seed.txt");

fn main() {
    let mut arguments = std::env::args().skip(1);
    let corpus_path = arguments
        .next()
        .expect("usage: evaluate_english_words <test.conllu> [snapshot.tsv]");
    let snapshot_path = arguments.next();
    let source = std::fs::read_to_string(corpus_path).expect("public English corpus");
    let words: BTreeSet<_> = source
        .lines()
        .filter_map(|line| {
            let mut fields = line.split('\t');
            fields.next()?.parse::<usize>().ok()?;
            let word = fields.next()?.to_ascii_lowercase();
            ((4..=64).contains(&word.len()) && word.bytes().all(|key| key.is_ascii_lowercase()))
                .then_some(word)
        })
        .collect();
    let hints: BTreeSet<_> = WORD_HINTS.split_ascii_whitespace().collect();
    let mut correct = 0;
    let mut total = 0;
    let mut unregistered_correct = 0;
    let mut unregistered_total = 0;
    let mut snapshot = String::new();
    let mut misses = BTreeSet::new();
    for word in words {
        let unregistered = !hints.contains(word.as_str());
        for context in ["", "gksrmf", "hello"] {
            let actual = decode_after(&word, context);
            correct += usize::from(actual == word);
            total += 1;
            if unregistered {
                unregistered_correct += usize::from(actual == word);
                unregistered_total += 1;
            }
            if actual != word {
                misses.insert(format!("{word} → {actual}"));
            }
            if snapshot_path.is_some() {
                snapshot.push_str(&format!("{context}\t{word}\t{actual}\n"));
            }
        }
    }
    println!("English test forms (4–64 ASCII letters, lowercase, 3 contexts): {correct}/{total}");
    println!("Forms absent from word hints: {unregistered_correct}/{unregistered_total}");
    for miss in misses.iter().take(20) {
        println!("  miss: {miss}");
    }
    if let Some(path) = snapshot_path {
        std::fs::write(path, snapshot).expect("write explicit corpus evaluation snapshot");
    }
}

fn decode_after(word: &str, context: &str) -> String {
    let mut session = Session::new(Mode::Auto).unwrap();
    for key in context.bytes() {
        session.process(Input::Letter(key));
    }
    session.process(Input::WordBoundary);
    for key in word.bytes() {
        session.process(Input::Letter(key));
    }
    session.process(Input::WordBoundary).commit
}
