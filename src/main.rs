// main.rs
pub mod dfa;
pub mod nfa;
pub mod pipeline;
pub mod regex;
pub mod types;
pub mod utils;

use crate::pipeline::tests::full_pipeline;
use crate::utils::dot::{dfa_to_dot, nfa_to_dot};

fn main() {
    let test_lang = "a*";

    println!("--- Pipeline Language: '{}' ---", test_lang);

    match full_pipeline("a*") {
        Ok(out) => {
            println!("ε-NFA:");
            // println!("{:#?}", out.enfa);
            println!("DOT for ε-NFA:\n{}", nfa_to_dot(&out.enfa));

            println!("\nNFA (ε removed):");
            // println!("{:#?}", out.nfa);
            println!("DOT for NFA:\n{}", nfa_to_dot(&out.nfa));

            println!("\nDFA:");
            // println!("{:#?}", out.dfa);
            println!("DOT for DFA:\n{}", dfa_to_dot(&out.dfa));

            println!("\nMinimized DFA:");
            // println!("{:#?}", out.mindfa);
            println!("DOT for minimized DFA:\n{}", dfa_to_dot(&out.mindfa));

            println!("\nAcceptance checks:");
            for s in ["", "a", "aa", "b", "ab"] {
                println!("  {:>2?}: {}", s, out.mindfa.accepts(s));
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
