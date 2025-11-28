// main.rs
mod types;
mod utils;

use types::Symbol;
use utils::graph_builder::GraphBuilder;

fn main() {
    println!("--- Running Tests ---");

    let mut gb = GraphBuilder::new();

    let s = gb.new_state();
    let e = gb.new_state();

    gb.add_transition(s, e, Symbol::Char('a'));
    gb.add_accept(e);

    let automaton = gb.build(s);

    println!("--- Constructed Automaton ---");
    println!("{:#?}", automaton);
}
