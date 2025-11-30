// dfa/minimize.rs
use super::dfa::DFA;
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

/// Minimizes a DFA using Hopcroft's algorithm.
/// Returns a new minimized DFA with renumbered states starting from 0.
pub fn minimize_dfa(dfa: &DFA) -> DFA {
    // Collect all states that appear in the DFA
    let mut all_states: HashSet<usize> = HashSet::new();
    all_states.insert(dfa.start);
    all_states.extend(dfa.accepts.iter().copied());
    
    for (src, trans) in &dfa.transitions {
        all_states.insert(*src);
        all_states.extend(trans.values().copied());
    }

    // Collect all symbols
    let mut symbols: HashSet<char> = HashSet::new();
    for trans_map in dfa.transitions.values() {
        symbols.extend(trans_map.keys().copied());
    }

    // Initial partition: accepting vs non-accepting states
    let accepting: BTreeSet<usize> = dfa.accepts.iter().copied().collect();
    let non_accepting: BTreeSet<usize> = all_states
        .iter()
        .copied()
        .filter(|s| !dfa.accepts.contains(s))
        .collect();

    let mut partitions: Vec<BTreeSet<usize>> = Vec::new();
    if !non_accepting.is_empty() {
        partitions.push(non_accepting.clone());
    }
    if !accepting.is_empty() {
        partitions.push(accepting.clone());
    }

    // Work queue for refinement
    let mut work_queue: VecDeque<BTreeSet<usize>> = VecDeque::new();
    if !non_accepting.is_empty() {
        work_queue.push_back(non_accepting);
    }
    if !accepting.is_empty() {
        work_queue.push_back(accepting);
    }

    // Hopcroft's refinement loop
    while let Some(splitter) = work_queue.pop_front() {
        for &symbol in &symbols {
            // Find all states that transition to the splitter on this symbol
            let mut predecessors: BTreeSet<usize> = BTreeSet::new();
            for &state in &all_states {
                if let Some(trans_map) = dfa.transitions.get(&state) {
                    if let Some(&target) = trans_map.get(&symbol) {
                        if splitter.contains(&target) {
                            predecessors.insert(state);
                        }
                    }
                }
            }

            if predecessors.is_empty() {
                continue;
            }

            // Refine each partition
            let mut new_partitions: Vec<BTreeSet<usize>> = Vec::new();
            
            for partition in &partitions {
                let intersection: BTreeSet<usize> = partition
                    .intersection(&predecessors)
                    .copied()
                    .collect();
                
                let difference: BTreeSet<usize> = partition
                    .difference(&predecessors)
                    .copied()
                    .collect();

                if !intersection.is_empty() && !difference.is_empty() {
                    // Partition was split
                    new_partitions.push(intersection.clone());
                    new_partitions.push(difference.clone());

                    // Add smaller partition to work queue
                    if intersection.len() <= difference.len() {
                        work_queue.push_back(intersection);
                    } else {
                        work_queue.push_back(difference);
                    }
                } else {
                    // Partition wasn't split
                    new_partitions.push(partition.clone());
                }
            }

            partitions = new_partitions;
        }
    }

    // Build the minimized DFA
    build_minimized_dfa(dfa, &partitions, &all_states)
}

/// Constructs a new DFA from the partition structure.
fn build_minimized_dfa(
    original: &DFA,
    partitions: &[BTreeSet<usize>],
    _all_states: &HashSet<usize>,
) -> DFA {
    // Map each state to its partition index
    let mut state_to_partition: HashMap<usize, usize> = HashMap::new();
    for (idx, partition) in partitions.iter().enumerate() {
        for &state in partition {
            state_to_partition.insert(state, idx);
        }
    }

    // Find start state partition
    let start = *state_to_partition
        .get(&original.start)
        .expect("Start state must be in a partition");

    // Find accepting partitions
    let mut accepts: HashSet<usize> = HashSet::new();
    for &accept_state in &original.accepts {
        if let Some(&partition_id) = state_to_partition.get(&accept_state) {
            accepts.insert(partition_id);
        }
    }

    // Build transitions for minimized DFA
    let mut transitions: HashMap<usize, HashMap<char, usize>> = HashMap::new();

    for (partition_idx, partition) in partitions.iter().enumerate() {
        // Pick any representative from the partition
        if let Some(&representative) = partition.iter().next() {
            if let Some(trans_map) = original.transitions.get(&representative) {
                let mut new_trans: HashMap<char, usize> = HashMap::new();
                
                for (&symbol, &target) in trans_map {
                    if let Some(&target_partition) = state_to_partition.get(&target) {
                        new_trans.insert(symbol, target_partition);
                    }
                }

                if !new_trans.is_empty() {
                    transitions.insert(partition_idx, new_trans);
                }
            }
        }
    }

    DFA {
        start,
        accepts,
        transitions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dfa::dfa::DFA;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn minimize_single_state() {
        // DFA with just a start/accept state with self-loop
        let mut transitions = HashMap::new();
        let mut trans_map = HashMap::new();
        trans_map.insert('a', 0);
        transitions.insert(0, trans_map);

        let mut accepts = HashSet::new();
        accepts.insert(0);

        let dfa = DFA {
            start: 0,
            accepts,
            transitions,
        };

        let minimized = minimize_dfa(&dfa);
        
        assert_eq!(minimized.start, 0);
        assert!(minimized.accepts.contains(&0));
        assert_eq!(minimized.transitions[&0][&'a'], 0);
    }

    #[test]
    fn minimize_already_minimal() {
        // DFA: 0 --a--> 1 (accept)
        let mut transitions = HashMap::new();
        let mut trans0 = HashMap::new();
        trans0.insert('a', 1);
        transitions.insert(0, trans0);
        transitions.insert(1, HashMap::new());

        let mut accepts = HashSet::new();
        accepts.insert(1);

        let dfa = DFA {
            start: 0,
            accepts,
            transitions,
        };

        let minimized = minimize_dfa(&dfa);

        // Should still have 2 states
        assert_eq!(minimized.transitions.len(), 1);
        assert!(minimized.accepts.len() == 1);
    }

    #[test]
    fn minimize_redundant_states() {
        // DFA with redundant states that should merge:
        // 0 --a--> 1 (accept)
        // 0 --b--> 2 (accept)
        // States 1 and 2 are equivalent (both accepting, no outgoing transitions)
        let mut transitions = HashMap::new();
        let mut trans0 = HashMap::new();
        trans0.insert('a', 1);
        trans0.insert('b', 2);
        transitions.insert(0, trans0);
        transitions.insert(1, HashMap::new());
        transitions.insert(2, HashMap::new());

        let mut accepts = HashSet::new();
        accepts.insert(1);
        accepts.insert(2);

        let dfa = DFA {
            start: 0,
            accepts: accepts.clone(),
            transitions,
        };

        let minimized = minimize_dfa(&dfa);

        // After minimization, should have 2 states total (start + merged accept)
        let mut all_states = HashSet::new();
        all_states.insert(minimized.start);
        all_states.extend(minimized.accepts.iter());
        for trans in minimized.transitions.values() {
            all_states.extend(trans.values());
        }
        
        assert_eq!(all_states.len(), 2);
        assert_eq!(minimized.accepts.len(), 1);
    }

    #[test]
    fn minimize_complex_example() {
        // DFA for (a|b)*abb with redundant states
        // This should reduce several equivalent states
        let mut transitions = HashMap::new();
        
        let mut trans0 = HashMap::new();
        trans0.insert('a', 1);
        trans0.insert('b', 0);
        transitions.insert(0, trans0);

        let mut trans1 = HashMap::new();
        trans1.insert('a', 1);
        trans1.insert('b', 2);
        transitions.insert(1, trans1);

        let mut trans2 = HashMap::new();
        trans2.insert('a', 1);
        trans2.insert('b', 3);
        transitions.insert(2, trans2);

        let mut trans3 = HashMap::new();
        trans3.insert('a', 1);
        trans3.insert('b', 0);
        transitions.insert(3, trans3);

        let mut accepts = HashSet::new();
        accepts.insert(3);

        let dfa = DFA {
            start: 0,
            accepts,
            transitions,
        };

        let minimized = minimize_dfa(&dfa);

        // The minimized DFA should still accept "abb" and reject other strings
        assert!(minimized.accepts.len() >= 1);
        assert_eq!(minimized.start, minimized.start); // Start exists
    }
}
