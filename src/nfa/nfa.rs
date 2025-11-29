// nfa/nfa.rs
use std::collections::HashMap;

pub type StateID = usize;

// ε-NFA edge: either a char-transition or epsilon
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TransitionLabel {
    Char(char),
    Epsilon,
}

#[derive(Debug, Clone)]
pub struct NFA {
    pub start: StateID,
    pub accept: Vec<StateID>,
    pub transitions: HashMap<StateID, Vec<(TransitionLabel, StateID)>>,
}

impl NFA {
    pub fn new() -> Self {
        Self {
            start: 0,
            accept: vec![],
            transitions: HashMap::new(),
        }
    }

    pub fn add_state(&mut self) -> StateID {
        let id = self.transitions.len();
        self.transitions.insert(id, Vec::new());
        id
    }

    pub fn add_edge(&mut self, from: StateID, label: TransitionLabel, to: StateID) {
        self.transitions.get_mut(&from).unwrap().push((label, to));
    }
}
