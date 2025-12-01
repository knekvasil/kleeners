// regex/parser.rs
use super::ast::RegexAST;
use super::tokenizer::{tokenize, Token};

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
            Some(t) if t == token => {
                self.consume();
                Ok(())
            }
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

/*
* =====================
*   HELPER FUNCTIONS
* =====================
*/

pub fn parse_language(input: &str) -> Result<RegexAST, ParseError> {
    let tokens = tokenize(input);
    let mut parser = Parser::new(tokens);

    let ast = parser.parse_expr()?;

    // Optional: ensure entire input was consumed
    if parser.peek().is_some() {
        return Err(ParseError::UnexpectedToken(parser.peek().unwrap().clone()));
    }

    Ok(ast)
}

/*
* =====================
*   CORRECTNESS TESTS
* =====================
*/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::regex::ast::RegexAST;
    use crate::regex::tokenizer::tokenize;

    fn parse(expr: &str) -> RegexAST {
        let tokens = tokenize(expr);
        let mut p = Parser::new(tokens);
        p.parse_expr().unwrap()
    }

    #[test]
    fn test_single_literal() {
        let ast = parse("a");
        assert!(matches!(ast, RegexAST::Char('a')));
    }

    #[test]
    fn test_concat() {
        let ast = parse("ab");
        match ast {
            RegexAST::Concat(a, b) => {
                assert!(matches!(*a, RegexAST::Char('a')));
                assert!(matches!(*b, RegexAST::Char('b')));
            }
            _ => panic!("expected Concat"),
        }
    }

    #[test]
    fn test_union() {
        let ast = parse("a+b");
        match ast {
            RegexAST::Union(a, b) => {
                assert!(matches!(*a, RegexAST::Char('a')));
                assert!(matches!(*b, RegexAST::Char('b')));
            }
            _ => panic!("expected Union"),
        }
    }

    #[test]
    fn test_kleene_star() {
        let ast = parse("a*");
        match ast {
            RegexAST::Star(inner) => {
                assert!(matches!(*inner, RegexAST::Char('a')));
            }
            _ => panic!("expected Star"),
        }
    }

    #[test]
    fn test_parentheses() {
        let ast = parse("(a+b)");
        match ast {
            RegexAST::Union(_, _) => {}
            _ => panic!("expected Union from (a+b)"),
        }
    }

    #[test]
    fn test_precedence_concat_vs_union() {
        // a+b c  must parse as Union(a, Concat(b, c))
        let ast = parse("a+bc");

        match ast {
            RegexAST::Union(left, right) => {
                assert!(matches!(*left, RegexAST::Char('a')));

                match *right {
                    RegexAST::Concat(x, y) => {
                        assert!(matches!(*x, RegexAST::Char('b')));
                        assert!(matches!(*y, RegexAST::Char('c')));
                    }
                    _ => panic!("expected bc to be concatenation"),
                }
            }
            _ => panic!("wrong precedence"),
        }
    }

    #[test]
    fn test_star_highest_precedence() {
        // ab* parses as Concat(a, b*)
        let ast = parse("ab*");

        match ast {
            RegexAST::Concat(left, right) => {
                assert!(matches!(*left, RegexAST::Char('a')));

                match *right {
                    RegexAST::Star(inner) => assert!(matches!(*inner, RegexAST::Char('b'))),
                    _ => panic!("expected b*"),
                }
            }
            _ => panic!("wrong precedence"),
        }
    }

    #[test]
    fn test_complex_expression() {
        let ast = parse("(a+b)*c");
        match ast {
            RegexAST::Concat(star_expr, c_literal) => {
                assert!(matches!(*c_literal, RegexAST::Char('c')));

                match *star_expr {
                    RegexAST::Star(inner) => {
                        assert!(matches!(*inner, RegexAST::Union(_, _)));
                    }
                    _ => panic!("expected (a+b)*"),
                }
            }
            _ => panic!("incorrect full expression parse"),
        }
    }
}
