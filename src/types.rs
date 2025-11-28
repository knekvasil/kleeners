// types.rs
use serde::{Deserialize, Serialize};

pub type StateID = u32;

/// A transition label (OR):
/// - `None`     = epsilon
/// - `Some(c)`  = char literal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Symbol {
    Epsilon,
    Char(char),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub from: StateID,
    pub to: StateID,
    pub symbol: Symbol,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Automaton {
    pub states: Vec<StateID>,
    pub start: StateID,
    pub accepts: Vec<StateID>,
    pub transitions: Vec<Transition>,
}
