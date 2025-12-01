// dfa/dfa.rs
use crate::nfa::nfa::{StateID, TransitionLabel, NFA};
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

pub struct DFA {
    pub start: usize,
    pub accepts: HashSet<usize>,
    pub transitions: HashMap<usize, HashMap<char, usize>>,
}

impl DFA {
    // Run the DFA on an input string.
    pub fn accepts(&self, input: &str) -> bool {
        let mut state = self.start;

        for ch in input.chars() {
            match self.transitions.get(&state).and_then(|m| m.get(&ch)) {
                Some(&next) => state = next,
                None => return false,
            }
        }

        self.accepts.contains(&state)
    }
}

pub fn nfa_to_dfa(nfa: &NFA) -> DFA {
    // Collect all symbols in the NFA w/ BTreeSet
    let mut symbols = BTreeSet::new();
    for edges in nfa.transitions.values() {
        for (label, _) in edges {
            if let TransitionLabel::Char(c) = label {
                symbols.insert(*c);
            }
        }
    }

    // --- Subset construction state machinery ---
    // Use BTreeSet as the key type since HashSet doesn't implement Hash
    let mut subset_to_id: HashMap<BTreeSet<usize>, usize> = HashMap::new();
    let mut id_to_subset: Vec<BTreeSet<StateID>> = Vec::new();
    let mut queue = VecDeque::new();
    let mut transitions: HashMap<usize, HashMap<char, usize>> = HashMap::new();
    let mut accepts = HashSet::new();

    // Start subset = { nfa.start }
    let start_subset: BTreeSet<StateID> = [nfa.start].into_iter().collect();

    subset_to_id.insert(start_subset.clone(), 0);
    id_to_subset.push(start_subset.clone());
    queue.push_back(0);

    // Check if start subset contains accept state
    if nfa.accept.iter().any(|a| start_subset.contains(a)) {
        accepts.insert(0);
    }

    // --- Main BFS over subsets ---
    while let Some(dfa_state) = queue.pop_front() {
        let subset = id_to_subset[dfa_state].clone();

        for &c in &symbols {
            // Compute target subset
            let mut target = BTreeSet::new();

            // NFA move
            for &s in &subset {
                if let Some(edges) = nfa.transitions.get(&s) {
                    for (label, dst) in edges {
                        if let TransitionLabel::Char(ch) = label {
                            if *ch == c {
                                target.insert(*dst);
                            }
                        }
                    }
                }
            }

            if target.is_empty() {
                continue;
            }

            // Does this subset already exist?
            let target_id = match subset_to_id.get(&target) {
                Some(id) => *id,
                None => {
                    let new_id = id_to_subset.len();
                    id_to_subset.push(target.clone());
                    subset_to_id.insert(target.clone(), new_id);
                    queue.push_back(new_id);

                    // If this new subset contains the NFA accept → record as accepting DFA state
                    if nfa.accept.iter().any(|a| target.contains(a)) {
                        accepts.insert(new_id);
                    }

                    new_id
                }
            };

            transitions
                .entry(dfa_state)
                .or_default()
                .insert(c, target_id);
        }
    }

    DFA {
        start: 0,
        accepts,
        transitions,
    }
}

/*
* =====================
*   CORRECTNESS TESTS
* =====================
*/

#[cfg(test)]
mod tests {
    use crate::dfa::dfa::nfa_to_dfa;
    use crate::nfa::nfa::{TransitionLabel, NFA};
    use std::collections::HashMap;

    #[test]
    fn dfa_from_single_char() {
        // NFA for "a"
        //
        // 0 --a--> 1
        let mut transitions = HashMap::new();
        transitions.insert(0, vec![(TransitionLabel::Char('a'), 1)]);
        transitions.insert(1, vec![]);

        let nfa = NFA {
            start: 0,
            accept: vec![1],
            transitions,
        };

        let dfa = nfa_to_dfa(&nfa);

        // DFA should have exactly two states
        assert_eq!(dfa.accepts.len(), 1);
        assert!(dfa.accepts.contains(&1));

        // Start state is 0
        assert_eq!(dfa.start, 0);

        // Transition: 0 -a-> 1
        assert_eq!(dfa.transitions.get(&0).unwrap().get(&'a'), Some(&1));
    }

    #[test]
    fn dfa_from_union() {
        // NFA for (a + b)
        //
        // 1 ->(a)-> 2
        // 0 ->(b)-> 3
        let mut transitions = HashMap::new();
        transitions.insert(
            0,
            vec![
                (TransitionLabel::Char('a'), 1),
                (TransitionLabel::Char('b'), 2),
            ],
        );
        transitions.insert(1, vec![]);
        transitions.insert(2, vec![]);

        let nfa = NFA {
            start: 0,
            accept: vec![1],
            transitions,
        };

        let dfa = nfa_to_dfa(&nfa);

        // Accepts should include the DFA state that corresponds to subset {1}
        assert!(dfa.accepts.contains(&1));

        // Transitions preserve union behavior deterministically.
        assert_eq!(dfa.transitions.get(&0).unwrap().get(&'a'), Some(&1));
        assert_eq!(dfa.transitions.get(&0).unwrap().get(&'b'), Some(&2));
    }

    #[test]
    fn dfa_from_concat_ab() {
        // NFA for "ab"
        //
        // 0 --a--> 1 --b--> 2
        let mut transitions = HashMap::new();
        transitions.insert(0, vec![(TransitionLabel::Char('a'), 1)]);
        transitions.insert(1, vec![(TransitionLabel::Char('b'), 2)]);
        transitions.insert(2, vec![]);

        let nfa = NFA {
            start: 0,
            accept: vec![2],
            transitions,
        };

        let dfa = nfa_to_dfa(&nfa);

        assert!(dfa.accepts.contains(&2)); // final state should accept

        // Deterministic verification:
        let s0 = dfa.start;
        let s1 = *dfa.transitions[&s0].get(&'a').unwrap();
        let s2 = *dfa.transitions[&s1].get(&'b').unwrap();

        assert!(dfa.accepts.contains(&s2));
    }

    #[test]
    fn dfa_from_star_a() {
        // NFA for a*
        //
        // 0 (accept) --a--> 0
        //
        let mut transitions = HashMap::new();
        transitions.insert(0, vec![(TransitionLabel::Char('a'), 0)]);

        let nfa = NFA {
            start: 0,
            accept: vec![0],
            transitions,
        };

        let dfa = nfa_to_dfa(&nfa);

        // Start state is accept
        assert!(dfa.accepts.contains(&dfa.start));

        // Transition loops on 'a'
        assert_eq!(dfa.transitions[&dfa.start].get(&'a'), Some(&dfa.start));
    }
    #[test]
    fn dfa_from_kleene_star_union_concat() {
        //
        // NFA for (a + b)*c
        //
        // loop: 0 --a--> 0 ...
        //       0 --b--> 0 ...
        //       0 --c--> 1   // accept
        //
        let mut transitions = HashMap::new();
        transitions.insert(
            0,
            vec![
                (TransitionLabel::Char('a'), 0),
                (TransitionLabel::Char('b'), 0),
                (TransitionLabel::Char('c'), 1),
            ],
        );
        transitions.insert(1, vec![]);

        let nfa = NFA {
            start: 0,
            accept: vec![1],
            transitions,
        };

        let dfa = nfa_to_dfa(&nfa);

        // DFA should accept after reading exactly one c after any a/b* prefix
        assert!(dfa.accepts.contains(&1));

        let s0 = dfa.start;
        let s_loop_a = dfa.transitions[&s0].get(&'a').copied().unwrap();
        let s_loop_b = dfa.transitions[&s0].get(&'b').copied().unwrap();
        let s_accept = dfa.transitions[&s0].get(&'c').copied().unwrap();

        assert_eq!(s_loop_a, s0);
        assert_eq!(s_loop_b, s0);
        assert!(dfa.accepts.contains(&s_accept));
    }
}
