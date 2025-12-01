// pipeline/tests.rs
use crate::dfa::dfa::nfa_to_dfa;
use crate::dfa::minimize::minimize_dfa;
use crate::nfa::epsilon_elimination::remove_epsilon;
use crate::nfa::thompson::enfa_from_ast;
use crate::regex::parser::parse_language;

use crate::dfa::dfa::DFA;
use crate::nfa::nfa::NFA;

/// The unified output of the entire pipeline.
pub struct FullOutput {
    pub enfa: NFA,
    pub nfa: NFA,
    pub dfa: DFA,
    pub mindfa: DFA,
}

/// Convert a language string into a minimized DFA.
pub fn full_pipeline(lang: &str) -> Result<FullOutput, String> {
    // 1. Parse the input language into an AST
    let ast = parse_language(lang).map_err(|e| format!("Parse error: {:?}", e))?;

    // 2. Thompson construction: AST → ε-NFA
    let enfa = enfa_from_ast(&ast);

    // 3. Eliminate ε-transitions: ENFA → NFA
    let nfa = remove_epsilon(&enfa);

    // 4. Subset construction: NFA → DFA
    let dfa = nfa_to_dfa(&nfa);

    // 5. Hopcroft (or equivalent): DFA → Minimized DFA
    let mindfa = minimize_dfa(&dfa);

    Ok(FullOutput {
        enfa,
        nfa,
        dfa,
        mindfa,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_language {
        ($min:expr, accept: [$($a:expr),*], reject: [$($r:expr),*]) => {
            $(
                assert!(
                    $min.accepts($a),
                    "Should accept {:?}", $a
                );
            )*
            $(
                assert!(
                    !$min.accepts($r),
                    "Should reject {:?}", $r
                );
            )*
        };
    }

    #[test]
    fn pipeline_atom_a() {
        let out = full_pipeline("a").unwrap();
        let m = out.mindfa;

        assert_language!(
            m,
            accept: ["a"],
            reject: ["", "b", "aa"]
        );
    }

    #[test]
    fn pipeline_union() {
        let out = full_pipeline("a+b").unwrap();
        let m = out.mindfa;

        assert_language!(
            m,
            accept: ["a", "b"],
            reject: ["", "ab", "ba"]
        );
    }

    #[test]
    fn pipeline_concat() {
        let out = full_pipeline("ab").unwrap();
        let m = out.mindfa;

        assert_language!(
            m,
            accept: ["ab"],
            reject: ["", "a", "b", "aba"]
        );
    }

    #[test]
    fn pipeline_star() {
        let out = full_pipeline("a*").unwrap();
        let m = out.mindfa;

        assert_language!(
            m,
            accept: ["", "a", "aa"],
            reject: ["b", "ab"]
        );
    }
}
