// regex/tokenizer.rs
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Char(char),
    Plus,   // +
    Star,   // *
    LParen, // (
    RParen, // )
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    for ch in input.chars() {
        match ch {
            '+' => tokens.push(Token::Plus),
            '*' => tokens.push(Token::Star),
            '(' => tokens.push(Token::LParen),
            ')' => tokens.push(Token::RParen),

            // treat anything alphanumeric as a literal
            c if c.is_alphanumeric() => tokens.push(Token::Char(c)),

            // tolerate whitespace
            c if c.is_whitespace() => continue,

            _ => panic!("Unexpected character in regex: {}", ch),
        }
    }

    tokens
}

/*
* =====================
*   CORRECTNESS TESTS
* =====================
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_literals() {
        let t = tokenize("abc");
        assert_eq!(
            t,
            vec![Token::Char('a'), Token::Char('b'), Token::Char('c'),]
        );
    }

    #[test]
    fn test_operators() {
        let t = tokenize("a+b*");
        assert_eq!(
            t,
            vec![Token::Char('a'), Token::Plus, Token::Char('b'), Token::Star,]
        );
    }

    #[test]
    fn test_parens() {
        let t = tokenize("(a+b)*c");
        assert_eq!(
            t,
            vec![
                Token::LParen,
                Token::Char('a'),
                Token::Plus,
                Token::Char('b'),
                Token::RParen,
                Token::Star,
                Token::Char('c'),
            ]
        );
    }
}
