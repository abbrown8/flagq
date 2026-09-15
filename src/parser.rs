use crate::error::SourceError;
use crate::lexer::{Token, TokenKind};

#[derive(Debug)]
pub struct Flag {
    pub name: String,
    pub default: bool,
    pub rules: Vec<Rule>,
}

/// A rule matches if any clause matches; a clause matches if all of its
/// conditions match. This is `or` of `and`s, so `or` reads as lower
/// precedence than `and` the way it does in most languages.
#[derive(Debug)]
pub struct Rule {
    pub effect: bool,
    pub clauses: Vec<Vec<Condition>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operator {
    Eq,
    NotEq,
    Lt,
    Le,
    Gt,
    Ge,
}

impl Operator {
    pub fn symbol(&self) -> &'static str {
        match self {
            Operator::Eq => "=",
            Operator::NotEq => "!=",
            Operator::Lt => "<",
            Operator::Le => "<=",
            Operator::Gt => ">",
            Operator::Ge => ">=",
        }
    }

    fn is_numeric(&self) -> bool {
        matches!(self, Operator::Lt | Operator::Le | Operator::Gt | Operator::Ge)
    }
}

#[derive(Debug)]
pub struct Condition {
    pub key: String,
    pub value: String,
    pub op: Operator,
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens[self.pos].clone();
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, kind: &TokenKind, context: &str) -> Result<Token, SourceError> {
        if std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind) {
            Ok(self.advance())
        } else {
            let tok = self.peek();
            Err(SourceError::new(
                tok.line,
                tok.col,
                format!(
                    "expected {} {}, found {}",
                    kind.describe(),
                    context,
                    tok.kind.describe()
                ),
            ))
        }
    }

    fn expect_ident(&mut self, expected: &str) -> Result<(), SourceError> {
        match &self.peek().kind {
            TokenKind::Ident(s) if s == expected => {
                self.advance();
                Ok(())
            }
            _ => {
                let tok = self.peek();
                Err(SourceError::new(
                    tok.line,
                    tok.col,
                    format!("expected `{}`, found {}", expected, tok.kind.describe()),
                ))
            }
        }
    }

    fn parse_bool_word(&mut self) -> Result<bool, SourceError> {
        let tok = self.advance();
        match &tok.kind {
            TokenKind::Ident(s) if s == "on" => Ok(true),
            TokenKind::Ident(s) if s == "off" => Ok(false),
            other => Err(SourceError::new(
                tok.line,
                tok.col,
                format!("expected `on` or `off`, found {}", other.describe()),
            )),
        }
    }

    fn parse_value(&mut self) -> Result<String, SourceError> {
        let tok = self.advance();
        match tok.kind {
            TokenKind::Str(s) => Ok(s),
            TokenKind::Ident(s) => Ok(s),
            other => Err(SourceError::new(
                tok.line,
                tok.col,
                format!("expected a value, found {}", other.describe()),
            )),
        }
    }

    fn parse_condition(&mut self) -> Result<Condition, SourceError> {
        let key_tok = self.advance();
        let key = match key_tok.kind {
            TokenKind::Ident(s) => s,
            other => {
                return Err(SourceError::new(
                    key_tok.line,
                    key_tok.col,
                    format!("expected a context key, found {}", other.describe()),
                ))
            }
        };
        let op = match &self.peek().kind {
            TokenKind::Equal => Operator::Eq,
            TokenKind::NotEqual => Operator::NotEq,
            TokenKind::Lt => Operator::Lt,
            TokenKind::Le => Operator::Le,
            TokenKind::Gt => Operator::Gt,
            TokenKind::Ge => Operator::Ge,
            _ => {
                let tok = self.peek();
                return Err(SourceError::new(
                    tok.line,
                    tok.col,
                    format!(
                        "expected `=`, `!=`, `<`, `<=`, `>` or `>=` after context key, found {}",
                        tok.kind.describe()
                    ),
                ));
            }
        };
        self.advance();
        let value_tok = self.peek().clone();
        let value = self.parse_value()?;
        if op.is_numeric() && value.parse::<f64>().is_err() {
            return Err(SourceError::new(
                value_tok.line,
                value_tok.col,
                format!(
                    "expected a number after `{}`, found `{}`",
                    op.symbol(),
                    value
                ),
            ));
        }
        Ok(Condition { key, value, op })
    }

    fn parse_and_group(&mut self) -> Result<Vec<Condition>, SourceError> {
        let mut conditions = vec![self.parse_condition()?];
        loop {
            match &self.peek().kind {
                TokenKind::Ident(s) if s == "and" => {
                    self.advance();
                    conditions.push(self.parse_condition()?);
                }
                _ => break,
            }
        }
        Ok(conditions)
    }

    fn parse_rule(&mut self) -> Result<Rule, SourceError> {
        self.expect(&TokenKind::Colon, "after `rule`")?;
        let effect = self.parse_bool_word()?;
        self.expect_ident("if")?;
        let mut clauses = vec![self.parse_and_group()?];
        loop {
            match &self.peek().kind {
                TokenKind::Ident(s) if s == "or" => {
                    self.advance();
                    clauses.push(self.parse_and_group()?);
                }
                _ => break,
            }
        }
        Ok(Rule { effect, clauses })
    }

    fn parse_flag(&mut self) -> Result<Flag, SourceError> {
        self.expect_ident("flag")?;
        let name_tok = self.advance();
        let name = match name_tok.kind {
            TokenKind::Ident(s) => s,
            other => {
                return Err(SourceError::new(
                    name_tok.line,
                    name_tok.col,
                    format!("expected a flag name, found {}", other.describe()),
                ))
            }
        };
        self.expect(&TokenKind::LBrace, "after flag name")?;

        self.expect_ident("default")?;
        self.expect(&TokenKind::Colon, "after `default`")?;
        let default = self.parse_bool_word()?;

        let mut rules = Vec::new();
        loop {
            match &self.peek().kind {
                TokenKind::Ident(s) if s == "rule" => {
                    self.advance();
                    rules.push(self.parse_rule()?);
                }
                TokenKind::RBrace => break,
                other => {
                    let tok = self.peek();
                    return Err(SourceError::new(
                        tok.line,
                        tok.col,
                        format!("expected `rule` or `}}`, found {}", other.describe()),
                    ));
                }
            }
        }
        self.expect(&TokenKind::RBrace, "to close flag block")?;

        Ok(Flag {
            name,
            default,
            rules,
        })
    }

    pub fn parse_file(mut self) -> Result<Vec<Flag>, SourceError> {
        let mut flags = Vec::new();
        while self.peek().kind != TokenKind::Eof {
            flags.push(self.parse_flag()?);
        }
        Ok(flags)
    }
}
