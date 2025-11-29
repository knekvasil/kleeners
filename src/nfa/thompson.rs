// nfa/thompson.rs
use super::nfa::{TransitionLabel, NFA};
use crate::regex::ast::RegexAST;

pub struct Thompson {
    pub nfa: NFA,
}

struct Fragment {
    start: usize,
    accept: usize,
}

impl Thompson {
    pub fn new() -> Self {
        Self { nfa: NFA::new() }
    }

    fn new_state(&mut self) -> usize {
        self.nfa.add_state()
    }

    fn add_epsilon(&mut self, from: usize, to: usize) {
        self.nfa.add_edge(from, TransitionLabel::Epsilon, to);
    }

    fn add_char(&mut self, from: usize, c: char, to: usize) {
        self.nfa.add_edge(from, TransitionLabel::Char(c), to);
    }

    pub fn from_ast(mut self, ast: &RegexAST) -> NFA {
        let frag = self.build(ast);
        self.nfa.start = frag.start;
        self.nfa.accept = frag.accept;
        self.nfa
    }

    fn build(&mut self, ast: &RegexAST) -> Fragment {
        match ast {
            RegexAST::Char(c) => self.char_frag(*c),
            RegexAST::Concat(a, b) => {
                let left = self.build(a);
                let right = self.build(b);

                // connect left.accept -> right.start via epsilon
                self.add_epsilon(left.accept, right.start);

                Fragment {
                    start: left.start,
                    accept: right.accept,
                }
            }
            RegexAST::Union(a, b) => {
                let left = self.build(a);
                let right = self.build(b);

                let s = self.new_state();
                let t = self.new_state();

                // s -> left.start
                self.add_epsilon(s, left.start);
                // s -> right.start
                self.add_epsilon(s, right.start);
                // left.accept -> t
                self.add_epsilon(left.accept, t);
                // right.accept -> t
                self.add_epsilon(right.accept, t);

                Fragment {
                    start: s,
                    accept: t,
                }
            }
            RegexAST::Star(expr) => {
                let inner = self.build(expr);

                let s = self.new_state();
                let t = self.new_state();

                // s -> inner.start
                self.add_epsilon(s, inner.start);
                // inner.accept -> inner.start
                self.add_epsilon(inner.accept, inner.start);
                // s -> t (skip)
                self.add_epsilon(s, t);
                // inner.accept -> t
                self.add_epsilon(inner.accept, t);

                Fragment {
                    start: s,
                    accept: t,
                }
            }
        }
    }

    fn char_frag(&mut self, c: char) -> Fragment {
        let s = self.new_state();
        let t = self.new_state();

        self.add_char(s, c, t);

        Fragment {
            start: s,
            accept: t,
        }
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

    fn build(expr: &str) -> NFA {
        let tokens = tokenize(expr);
        let mut parser = Parser::new(tokens);
        let ast = parser.parse_expr().unwrap();

        Thompson::new().from_ast(&ast)
    }

    #[test]
    fn test_single_char_nfa() {
        let nfa = build("a");
        assert_eq!(nfa.transitions.len(), 2); // 2 states for 'a'

        let outgoing = nfa.transitions.get(&nfa.start).unwrap();
        assert_eq!(outgoing.len(), 1);

        assert!(matches!(outgoing[0].0, TransitionLabel::Char('a')));
        assert_eq!(outgoing[0].1, nfa.accept);
    }

    #[test]
    fn test_concat_nfa() {
        let nfa = build("ab");
        assert!(nfa.transitions.len() >= 4);

        // There must be an epsilon connecting a.accept → b.start
        let mut epsilon_found = false;

        for edges in nfa.transitions.values() {
            for (label, _) in edges {
                if *label == TransitionLabel::Epsilon {
                    epsilon_found = true;
                }
            }
        }

        assert!(
            epsilon_found,
            "expected epsilon between concatenated fragments"
        );
    }

    #[test]
    fn test_union_nfa() {
        let nfa = build("a+b");

        // The union creates a new start & new accept
        assert!(nfa.transitions.len() >= 4);

        let start_edges = nfa.transitions.get(&nfa.start).unwrap();
        assert_eq!(start_edges.len(), 2);
        assert!(start_edges.iter().all(|e| e.0 == TransitionLabel::Epsilon));
    }

    #[test]
    fn test_star_nfa() {
        let nfa = build("a*");

        // Should have epsilon transitions supporting:
        // loop: inner.accept → inner.start
        // bypass: start → accept

        let mut epsilons = 0;

        for edges in nfa.transitions.values() {
            epsilons += edges
                .iter()
                .filter(|(label, _)| *label == TransitionLabel::Epsilon)
                .count();
        }

        assert!(
            epsilons >= 3,
            "expected multiple epsilon transitions for star"
        );
    }

    #[test]
    fn test_complex_nfa() {
        let nfa = build("(a+b)*c");

        assert!(nfa.transitions.len() >= 8);
        assert!(nfa.start < nfa.accept);
    }
}
