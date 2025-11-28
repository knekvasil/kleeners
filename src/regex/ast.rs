// regest/ast.rs
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegexAST {
    // A single char literal
    Char(char),
    // Concatenation: AB
    Concat(Box<RegexAST>, Box<RegexAST>),
    // Union: (A + B)
    Union(Box<RegexAST>, Box<RegexAST>),
    // Kleene star: (A*)
    Star(Box<RegexAST>),
}

