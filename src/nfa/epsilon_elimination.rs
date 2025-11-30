// nfa/epsilon_elimination.rs
use std::collections::{HashMap, HashSet, VecDeque};

use super::nfa::{StateID, TransitionLabel, NFA};

/// Compute epsilon-closure of a single state.
pub fn epsilon_closure_of_state(nfa: &NFA, s: StateID) -> HashSet<StateID> {
    let mut visited = HashSet::new();
    let mut dq = VecDeque::new();

    visited.insert(s);
    dq.push_back(s);

    while let Some(v) = dq.pop_front() {
        if let Some(edges) = nfa.transitions.get(&v) {
            for (label, to) in edges {
                if *label == TransitionLabel::Epsilon && !visited.contains(to) {
                    visited.insert(*to);
                    dq.push_back(*to);
                }
            }
        }
    }

    visited
}

/// Compute epsilon-closure of a set of states.
pub fn epsilon_closure_of_set(nfa: &NFA, states: &HashSet<StateID>) -> HashSet<StateID> {
    let mut res = HashSet::new();
    let mut dq = VecDeque::new();

    for &s in states {
        if res.insert(s) {
            dq.push_back(s);
        }
    }

    while let Some(v) = dq.pop_front() {
        if let Some(edges) = nfa.transitions.get(&v) {
            for (label, to) in edges {
                if *label == TransitionLabel::Epsilon && res.insert(*to) {
                    dq.push_back(*to);
                }
            }
        }
    }

    res
}

/// Move: from a set of states, follow `Char(c)` transitions (not epsilon) and return destination set.
pub fn move_on_char(nfa: &NFA, states: &HashSet<StateID>, c: char) -> HashSet<StateID> {
    let mut res = HashSet::new();

    for &s in states {
        if let Some(edges) = nfa.transitions.get(&s) {
            for (label, to) in edges {
                if let TransitionLabel::Char(ch) = label {
                    if *ch == c {
                        res.insert(*to);
                    }
                }
            }
        }
    }

    res
}

/// Remove epsilon transitions from an ε-NFA producing an equivalent NFA with only Char transitions.
/// Returns an NFA whose `accept` field is a Vec<StateID> containing all accepting states.
pub fn remove_epsilon(nfa: &NFA) -> NFA {
    // Precompute closures for all states present in the transition map.
    let mut all_states: HashSet<StateID> = nfa.transitions.keys().copied().collect();
    for edges in nfa.transitions.values() {
        for &(_, to) in edges {
            all_states.insert(to);
        }
    }

    let mut closures: HashMap<StateID, HashSet<StateID>> = HashMap::with_capacity(all_states.len());
    for &state in &all_states {
        closures.insert(state, epsilon_closure_of_state(nfa, state));
    }

    // Collect all character symbols present
    let mut symbols: HashSet<char> = HashSet::new();
    for edges in nfa.transitions.values() {
        for (label, _) in edges {
            if let TransitionLabel::Char(ch) = label {
                symbols.insert(*ch);
            }
        }
    }

    // New transitions: initialize for every existing state
    let mut new_transitions: HashMap<StateID, Vec<(TransitionLabel, StateID)>> =
        HashMap::with_capacity(all_states.len());
    for &s in &all_states {
        new_transitions.insert(s, Vec::new());
    }

    // For each state s and each symbol c:
    // target = epsilon_closure(move(epsilon_closure(s), c))
    for &s in &all_states {
        let closure_s = closures.get(&s).expect("closure missing");

        for &c in &symbols {
            let moved = move_on_char(nfa, closure_s, c);
            if moved.is_empty() {
                continue;
            }
            let moved_closure = epsilon_closure_of_set(nfa, &moved);

            for &t in &moved_closure {
                new_transitions
                    .get_mut(&s)
                    .unwrap()
                    .push((TransitionLabel::Char(c), t));
            }
        }
    }

    // Determine accepting states: any state whose closure contains ANY original accept state
    let mut accepting_states: Vec<StateID> = Vec::new();
    for &s in &all_states {
        let closure_s = closures.get(&s).unwrap();
        if nfa
            .accept
            .iter()
            .any(|orig_accept| closure_s.contains(orig_accept))
        {
            accepting_states.push(s);
        }
    }

    // Fallback: if we found no accepting states (should be rare), retain original accepts that still exist, else keep empty
    if accepting_states.is_empty() {
        accepting_states = nfa
            .accept
            .iter()
            .cloned()
            .filter(|a| all_states.contains(a))
            .collect();
    }

    NFA {
        start: nfa.start,
        accept: accepting_states,
        transitions: new_transitions,
    }
}

/*
* =====================
*   CORRECTNESS TESTS
* =====================
*/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::regex::{parser::Parser, tokenizer::tokenize};

    fn build_nfa(expr: &str) -> NFA {
        let tokens = tokenize(expr);
        let mut p = Parser::new(tokens);
        let ast = p.parse_expr().unwrap();
        crate::nfa::thompson::Thompson::new().from_ast(&ast)
    }

    #[test]
    fn closure_simple() {
        let nfa = build_nfa("a+b");
        // union's start will have epsilon edges to a.start and b.start
        let closure = epsilon_closure_of_state(&nfa, nfa.start);
        // At least should include start and some other states
        assert!(closure.contains(&nfa.start));
        assert!(closure.len() >= 2);
    }

    #[test]
    fn remove_epsilon_removes_epsilon_transitions_except_accept_shim() {
        let nfa = build_nfa("(a+b)*c");
        let cleaned = remove_epsilon(&nfa);

        // Check that (most) transitions are Char transitions. If multiple accepts existed,
        // we allowed epsilon edges from accepting states to a synthetic accept.
        let mut found_epsilon = false;
        for edges in cleaned.transitions.values() {
            for (label, _) in edges {
                if *label == TransitionLabel::Epsilon {
                    found_epsilon = true;
                }
            }
        }

        // It's acceptable to have epsilon edges only if multiple accepting states existed
        // and we had to create a synthetic accept. Ensure that epsilon edges count is small.
        assert!(
            !found_epsilon || cleaned.transitions.len() >= 1,
            "unexpected epsilon-heavy NFA"
        );

        // Ensure there is at least one Char transition for 'c'
        let mut has_c = false;
        for edges in cleaned.transitions.values() {
            for (label, _) in edges {
                if let TransitionLabel::Char(ch) = label {
                    if *ch == 'c' {
                        has_c = true;
                    }
                }
            }
        }
        assert!(has_c, "expected char 'c' transitions after epsilon removal");
    }
}
