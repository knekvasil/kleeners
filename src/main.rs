// main.rs
pub mod nfa;
pub mod regex;
pub mod types;
pub mod utils;

use types::Symbol;
use utils::graph_builder::GraphBuilder;

fn main() {
    println!("--- Running Playground ---");

    let mut gb = GraphBuilder::new();

    let s = gb.new_state();
    let e = gb.new_state();

    gb.add_transition(s, e, Symbol::Char('a'));
    gb.add_accept(e);

    let automaton = gb.build(s);

    println!("--- Constructed Automaton ---");
    println!("{:#?}", automaton);
}
