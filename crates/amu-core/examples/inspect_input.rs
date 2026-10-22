use amu_core::{Input, Mode, Session};

fn main() {
    for input in std::env::args().skip(1) {
        let mut automatic = Session::new(Mode::Auto).unwrap();
        let mut korean = Session::new(Mode::Korean).unwrap();
        println!("input: {input}");
        for keys in input.split_ascii_whitespace() {
            for key in keys.bytes() {
                automatic.process(Input::Letter(key));
                korean.process(Input::Letter(key));
            }
            let selected = automatic.process(Input::WordBoundary);
            let composed = korean.process(Input::WordBoundary);
            println!(
                "  {keys}: {:?} → {}; Korean candidate: {}",
                selected.language, selected.commit, composed.commit
            );
        }
    }
}
