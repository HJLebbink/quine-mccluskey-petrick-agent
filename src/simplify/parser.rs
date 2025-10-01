// Simple Boolean expression parser (mini-MVP version)

use super::types::BoolExpr;

/// Parse a simple Boolean expression string
/// Supports: variables (a-z), &&, ||, !, parentheses
///
/// Examples:
/// - "a" → Var("a")
/// - "!a" → Not(Var("a"))
/// - "a && b" → And(Var("a"), Var("b"))
/// - "a || b && c" → Or(Var("a"), And(Var("b"), Var("c")))
pub fn parse_bool_expr(input: &str) -> Result<BoolExpr, String> {
    let tokens = tokenize(input)?;
    let mut parser = Parser::new(tokens);
    parser.parse_or()
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Var(String),
    And,
    Or,
    Not,
    LParen,
    RParen,
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' | '\n' | '\r' => {
                chars.next();
            }
            '(' => {
                tokens.push(Token::LParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RParen);
                chars.next();
            }
            '!' => {
                tokens.push(Token::Not);
                chars.next();
            }
            '&' => {
                chars.next();
                if chars.peek() == Some(&'&') {
                    chars.next();
                    tokens.push(Token::And);
                } else {
                    return Err("Expected '&&', found single '&'".to_string());
                }
            }
            '|' => {
                chars.next();
                if chars.peek() == Some(&'|') {
                    chars.next();
                    tokens.push(Token::Or);
                } else {
                    return Err("Expected '||', found single '|'".to_string());
                }
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut var_name = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '_' {
                        var_name.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Var(var_name));
            }
            _ => {
                return Err(format!("Unexpected character: '{}'", ch));
            }
        }
    }

    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn parse_or(&mut self) -> Result<BoolExpr, String> {
        let mut left = self.parse_and()?;

        while let Some(Token::Or) = self.current() {
            self.advance();
            let right = self.parse_and()?;
            left = BoolExpr::or(left, right);
        }

        Ok(left)
    }

    fn parse_and(&mut self) -> Result<BoolExpr, String> {
        let mut left = self.parse_not()?;

        while let Some(Token::And) = self.current() {
            self.advance();
            let right = self.parse_not()?;
            left = BoolExpr::and(left, right);
        }

        Ok(left)
    }

    fn parse_not(&mut self) -> Result<BoolExpr, String> {
        if let Some(Token::Not) = self.current() {
            self.advance();
            let expr = self.parse_not()?;
            Ok(BoolExpr::not(expr))
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Result<BoolExpr, String> {
        match self.current() {
            Some(Token::Var(name)) => {
                let expr = BoolExpr::var(name);
                self.advance();
                Ok(expr)
            }
            Some(Token::LParen) => {
                self.advance();
                let expr = self.parse_or()?;
                if let Some(Token::RParen) = self.current() {
                    self.advance();
                    Ok(expr)
                } else {
                    Err("Expected ')'".to_string())
                }
            }
            Some(token) => Err(format!("Unexpected token: {:?}", token)),
            None => Err("Unexpected end of input".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_variable() {
        let expr = parse_bool_expr("a").unwrap();
        assert_eq!(expr, BoolExpr::var("a"));
    }

    #[test]
    fn test_parse_not() {
        let expr = parse_bool_expr("!a").unwrap();
        assert_eq!(expr, BoolExpr::not(BoolExpr::var("a")));
    }

    #[test]
    fn test_parse_and() {
        let expr = parse_bool_expr("a && b").unwrap();
        assert_eq!(expr, BoolExpr::and(BoolExpr::var("a"), BoolExpr::var("b")));
    }

    #[test]
    fn test_parse_or() {
        let expr = parse_bool_expr("a || b").unwrap();
        assert_eq!(expr, BoolExpr::or(BoolExpr::var("a"), BoolExpr::var("b")));
    }

    #[test]
    fn test_parse_complex() {
        let expr = parse_bool_expr("a && b || c").unwrap();
        // Should be: (a && b) || c
        let expected = BoolExpr::or(
            BoolExpr::and(BoolExpr::var("a"), BoolExpr::var("b")),
            BoolExpr::var("c"),
        );
        assert_eq!(expr, expected);
    }

    #[test]
    fn test_parse_parentheses() {
        let expr = parse_bool_expr("a && (b || c)").unwrap();
        let expected = BoolExpr::and(
            BoolExpr::var("a"),
            BoolExpr::or(BoolExpr::var("b"), BoolExpr::var("c")),
        );
        assert_eq!(expr, expected);
    }

    #[test]
    fn test_parse_double_not() {
        let expr = parse_bool_expr("!!a").unwrap();
        assert_eq!(
            expr,
            BoolExpr::not(BoolExpr::not(BoolExpr::var("a")))
        );
    }
}
