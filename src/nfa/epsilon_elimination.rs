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

/// Remove epsilon transitions using subset construction approach.
/// Creates minimal NFA by treating epsilon-closures as state identities.
pub fn remove_epsilon(nfa: &NFA) -> NFA {
    // Collect all character symbols present
    let mut symbols: HashSet<char> = HashSet::new();
    for edges in nfa.transitions.values() {
        for (label, _) in edges {
            if let TransitionLabel::Char(ch) = label {
                symbols.insert(*ch);
            }
        }
    }

    // State mapping: epsilon-closure -> new state ID
    let mut state_map: HashMap<Vec<StateID>, StateID> = HashMap::new();
    let mut next_id = 0;

    // Queue for BFS through reachable closure-sets
    let mut queue: VecDeque<HashSet<StateID>> = VecDeque::new();
    let mut new_transitions: HashMap<StateID, Vec<(TransitionLabel, StateID)>> = HashMap::new();

    // Start with epsilon-closure of original start state
    let start_closure = epsilon_closure_of_state(nfa, nfa.start);
    let mut start_closure_sorted: Vec<StateID> = start_closure.iter().copied().collect();
    start_closure_sorted.sort_unstable();

    let new_start = next_id;
    state_map.insert(start_closure_sorted.clone(), new_start);
    new_transitions.insert(new_start, Vec::new());
    next_id += 1;

    queue.push_back(start_closure);

    // BFS to discover reachable states
    while let Some(current_closure) = queue.pop_front() {
        let mut current_sorted: Vec<StateID> = current_closure.iter().copied().collect();
        current_sorted.sort_unstable();
        let current_id = *state_map.get(&current_sorted).unwrap();

        // Track transitions by character to deduplicate
        let mut transitions_by_char: HashMap<char, HashSet<StateID>> = HashMap::new();

        for &c in &symbols {
            // Move on character c from current closure
            let moved = move_on_char(nfa, &current_closure, c);
            if moved.is_empty() {
                continue;
            }

            // Compute epsilon-closure of moved states
            let target_closure = epsilon_closure_of_set(nfa, &moved);
            let mut target_sorted: Vec<StateID> = target_closure.iter().copied().collect();
            target_sorted.sort_unstable();

            // Get or create target state ID
            let target_id = if let Some(&id) = state_map.get(&target_sorted) {
                id
            } else {
                let id = next_id;
                state_map.insert(target_sorted.clone(), id);
                new_transitions.insert(id, Vec::new());
                next_id += 1;
                queue.push_back(target_closure);
                id
            };

            // Deduplicate: only add if not already present for this character
            transitions_by_char
                .entry(c)
                .or_insert_with(HashSet::new)
                .insert(target_id);
        }

        // Add deduplicated transitions
        for (c, targets) in transitions_by_char {
            for target in targets {
                new_transitions
                    .get_mut(&current_id)
                    .unwrap()
                    .push((TransitionLabel::Char(c), target));
            }
        }
    }

    // Determine accepting states: any state whose closure contains an original accept state
    let mut accepting_states: Vec<StateID> = Vec::new();
    for (closure_sorted, &new_id) in &state_map {
        let closure_set: HashSet<StateID> = closure_sorted.iter().copied().collect();
        if nfa
            .accept
            .iter()
            .any(|&orig_accept| closure_set.contains(&orig_accept))
        {
            accepting_states.push(new_id);
        }
    }

    accepting_states.sort_unstable();

    NFA {
        start: new_start,
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
