// utils/dot.rs
use crate::dfa::dfa::DFA;
use crate::nfa::nfa::NFA;

fn escape(c: char) -> String {
    match c {
        '"' => "\\\"".into(),
        _ => c.to_string()
    }
}

pub fn nfa_to_dot(nfa: &NFA) -> String {
    let mut out = String::new();
    out.push_str("digraph NFA {\n  rankdir=LR;\n  node [shape=circle];\n");

    out.push_str(&format!("  start [shape=point];\n  start -> {};\n", nfa.start));

    for (src, edges) in &nfa.transitions {
        for (label, dst) in edges {
            let lbl = match label {
                crate::nfa::nfa::TransitionLabel::Char(c) => escape(*c),
                crate::nfa::nfa::TransitionLabel::Epsilon => "ε".into(),
            };
            out.push_str(&format!("  {} -> {} [label=\"{}\"];\n", src, dst, lbl));
        }
    }

    for a in &nfa.accept {
        out.push_str(&format!("  {} [shape=doublecircle];\n", a));
    }

    out.push_str("}\n");
    out
}

pub fn dfa_to_dot(dfa: &DFA) -> String {
    let mut out = String::new();
    out.push_str("digraph DFA {\n  rankdir=LR;\n  node [shape=circle];\n");

    out.push_str(&format!("  start [shape=point];\n  start -> {};\n", dfa.start));

    for (src, map) in &dfa.transitions {
        for (c, dst) in map {
            out.push_str(&format!("  {} -> {} [label=\"{}\"];\n", src, dst, escape(*c)));
        }
    }

    for a in &dfa.accepts {
        out.push_str(&format!("  {} [shape=doublecircle];\n", a));
    }

    out.push_str("}\n");
    out
}
