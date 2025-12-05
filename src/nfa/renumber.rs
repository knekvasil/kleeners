// nfa/renumber.rs
use std::collections::{HashMap, VecDeque};
use super::nfa::{StateID, NFA};


// Renumber states in DFS order from the start state.
// This tends to follow the "natural flow" of the regex more closely.
pub fn renumber_dfs(nfa: &NFA) -> NFA {
    let mut old_to_new: HashMap<StateID, StateID> = HashMap::new();
    let mut stack: Vec<StateID> = Vec::new();
    let mut next_id = 0;
    
    // Start state becomes 0
    old_to_new.insert(nfa.start, next_id);
    stack.push(nfa.start);
    next_id += 1;
    
    // DFS traversal
    while let Some(current) = stack.pop() {
        if let Some(edges) = nfa.transitions.get(&current) {
            // Process in reverse order so first edge is explored first
            for (_, to) in edges.iter().rev() {
                if !old_to_new.contains_key(to) {
                    old_to_new.insert(*to, next_id);
                    stack.push(*to);
                    next_id += 1;
                }
            }
        }
    }
    
    // Rebuild NFA with new numbering
    let mut new_transitions = HashMap::new();
    for (old_state, edges) in &nfa.transitions {
        if let Some(&new_state) = old_to_new.get(old_state) {
            let new_edges = edges
                .iter()
                .filter_map(|(label, old_to)| {
                    old_to_new.get(old_to).map(|&new_to| (label.clone(), new_to))
                })
                .collect();
            new_transitions.insert(new_state, new_edges);
        }
    }
    
    let new_accept = nfa
        .accept
        .iter()
        .filter_map(|old| old_to_new.get(old).copied())
        .collect();
    
    NFA {
        start: 0,
        accept: new_accept,
        transitions: new_transitions,
    }
}

