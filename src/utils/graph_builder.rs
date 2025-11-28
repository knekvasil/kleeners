// utils/graph_builder.rs
use crate::types::{Automaton, StateID, Symbol, Transition};

pub struct GraphBuilder {
    next_state: StateID,
    transitions: Vec<Transition>,
    accepts: Vec<StateID>,
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            next_state: 0,
            transitions: Vec::new(),
            accepts: Vec::new(),
        }
    }

    // Add new state.
    #[inline]
    pub fn new_state(&mut self) -> StateID {
        let id = self.next_state;
        self.next_state += 1;
        id
    }

    // Add a transition between states.
    #[inline]
    pub fn add_transition(&mut self, from: StateID, to: StateID, symbol: Symbol) {
        self.transitions.push(Transition { from, to, symbol });
    }

    // Add an accepting state.
    #[inline]
    pub fn add_accept(&mut self, state: StateID) {
        self.accepts.push(state);
    }

    pub fn build(self, start: StateID) -> Automaton {
        let states: Vec<StateID> = (0..self.next_state).collect();

        Automaton {
            states,
            start,
            accepts: self.accepts,
            transitions: self.transitions,
        }
    }
}
