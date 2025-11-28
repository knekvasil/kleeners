// regex/tokenizer.rs
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Char(char),
    Plus,      // +
    Star,      // *
    LParen,    // (
    RParen,    // )
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
