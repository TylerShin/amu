use amu_core::{DecisionStatus, Input, Mode, Session};
use std::collections::BTreeSet;

fn main() {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    assert_eq!(
        paths.len(),
        2,
        "usage: evaluate_short_latin <train.conllu> <test.conllu>"
    );
    let training = consonant_forms(&paths[0]);
    let test = consonant_forms(&paths[1]);
    for (name, forms) in [
        ("all test forms", test.clone()),
        (
            "forms absent from training",
            test.difference(&training).cloned().collect(),
        ),
    ] {
        let mut correct = 0;
        let mut uncertain = 0;
        let mut total = 0;
        let mut misses = BTreeSet::new();
        for word in forms {
            for context in ["", "gksrmf", "hello"] {
                let mut session = Session::new(Mode::Auto).unwrap();
                if !context.is_empty() {
                    for key in context.bytes() {
                        session.process(Input::Letter(key));
                    }
                    session.process(Input::WordBoundary);
                }
                for key in word.bytes() {
                    session.process(Input::Letter(key));
                }
                uncertain +=
                    usize::from(session.interpretation().status == DecisionStatus::Ambiguous);
                let actual = session.process(Input::WordBoundary).commit;
                correct += usize::from(actual == word);
                total += 1;
                if actual != word {
                    misses.insert(format!("{word} → {actual}"));
                }
            }
        }
        println!("{name}, lowercased across 3 contexts: {correct}/{total}; uncertain={uncertain}");
        for miss in misses {
            println!("  miss: {miss}");
        }
    }
}

fn consonant_forms(path: &str) -> BTreeSet<String> {
    let source = std::fs::read_to_string(path).expect("public corpus file");
    source
        .lines()
        .filter_map(|line| {
            let mut fields = line.split('\t');
            fields.next()?.parse::<usize>().ok()?;
            let word = fields.next()?.to_ascii_lowercase();
            ((2..=6).contains(&word.len())
                && word
                    .bytes()
                    .all(|key| key.is_ascii_lowercase() && !b"aeiouy".contains(&key)))
            .then_some(word)
        })
        .collect()
}
