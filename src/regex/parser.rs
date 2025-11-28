// regex/parser.rs
use crate::regex::ast::RegexAST;
use crate::regex::tokenizer::Token;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedEnd,
    UnexpectedToken(Token),
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn consume(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let t = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(t)
        } else {
            None
        }
    }

    fn expect(&mut self, token: &Token) -> Result<(), ParseError> {
        match self.peek() {
            Some(t) if t == token => { self.consume(); Ok(()) }
            Some(t) => Err(ParseError::UnexpectedToken(t.clone())),
            None => Err(ParseError::UnexpectedEnd),
        }
    }

    // Grammar:
    // Union: '+'
    pub fn parse_expr(&mut self) -> Result<RegexAST, ParseError> {
        let mut node = self.parse_term()?;

        while let Some(Token::Plus) = self.peek() {
            self.consume(); // consume '+'
            let rhs = self.parse_term()?;
            node = RegexAST::Union(Box::new(node), Box::new(rhs));
        }

        Ok(node)
    }

    // Term: 'char'
    pub fn parse_term(&mut self) -> Result<RegexAST, ParseError> {
        let mut node = self.parse_factor()?;

        loop {
            match self.peek() {
                Some(Token::Char(_)) | Some(Token::LParen) => {
                    let rhs = self.parse_factor()?;
                    node = RegexAST::Concat(Box::new(node), Box::new(rhs));
                }
                _ => break,
            }
        }

        Ok(node)
    }

    // Star: '*' 
    pub fn parse_factor(&mut self) -> Result<RegexAST, ParseError> {
        let mut node = self.parse_primary()?;

        while let Some(Token::Star) = self.peek() {
            self.consume();
            node = RegexAST::Star(Box::new(node));
        }

        Ok(node)
    }

    // Paren: '(', ')' [Recursively]
    pub fn parse_primary(&mut self) -> Result<RegexAST, ParseError> {
        match self.consume() {
            Some(Token::Char(c)) => Ok(RegexAST::Char(c)),
            Some(Token::LParen) => {
                let node = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(node)
            }
            Some(t) => Err(ParseError::UnexpectedToken(t)),
            None => Err(ParseError::UnexpectedEnd),
        }
    }
}
