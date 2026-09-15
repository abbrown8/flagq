use crate::error::SourceError;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Ident(String),
    Str(String),
    LBrace,
    RBrace,
    Colon,
    Equal,
    NotEqual,
    Lt,
    Le,
    Gt,
    Ge,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}

impl TokenKind {
    pub fn describe(&self) -> String {
        match self {
            TokenKind::Ident(s) => format!("`{}`", s),
            TokenKind::Str(s) => format!("\"{}\"", s),
            TokenKind::LBrace => "`{`".to_string(),
            TokenKind::RBrace => "`}`".to_string(),
            TokenKind::Colon => "`:`".to_string(),
            TokenKind::Equal => "`=`".to_string(),
            TokenKind::NotEqual => "`!=`".to_string(),
            TokenKind::Lt => "`<`".to_string(),
            TokenKind::Le => "`<=`".to_string(),
            TokenKind::Gt => "`>`".to_string(),
            TokenKind::Ge => "`>=`".to_string(),
            TokenKind::Eof => "end of file".to_string(),
        }
    }
}

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '-' || c == '_'
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            chars: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    fn skip_trivia(&mut self) {
        loop {
            match self.peek() {
                Some(c) if c.is_whitespace() => {
                    self.advance();
                }
                Some('#') => {
                    while let Some(c) = self.peek() {
                        if c == '\n' {
                            break;
                        }
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>, SourceError> {
        let mut tokens = Vec::new();
        loop {
            self.skip_trivia();
            let (line, col) = (self.line, self.col);
            let Some(c) = self.peek() else {
                tokens.push(Token {
                    kind: TokenKind::Eof,
                    line,
                    col,
                });
                break;
            };

            let kind = match c {
                '{' => {
                    self.advance();
                    TokenKind::LBrace
                }
                '}' => {
                    self.advance();
                    TokenKind::RBrace
                }
                ':' => {
                    self.advance();
                    TokenKind::Colon
                }
                '=' => {
                    self.advance();
                    TokenKind::Equal
                }
                '!' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        TokenKind::NotEqual
                    } else {
                        return Err(SourceError::new(
                            line,
                            col,
                            "expected `=` after `!`",
                        ));
                    }
                }
                '<' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        TokenKind::Le
                    } else {
                        TokenKind::Lt
                    }
                }
                '>' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        TokenKind::Ge
                    } else {
                        TokenKind::Gt
                    }
                }
                '"' => self.read_string(line, col)?,
                c if is_ident_char(c) => self.read_ident(),
                other => {
                    return Err(SourceError::new(
                        line,
                        col,
                        format!("unexpected character '{}'", other),
                    ));
                }
            };

            tokens.push(Token { kind, line, col });
        }
        Ok(tokens)
    }

    fn read_ident(&mut self) -> TokenKind {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if is_ident_char(c) {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }
        TokenKind::Ident(s)
    }

    fn read_string(&mut self, start_line: usize, start_col: usize) -> Result<TokenKind, SourceError> {
        self.advance(); // opening quote
        let mut s = String::new();
        loop {
            match self.advance() {
                Some('"') => return Ok(TokenKind::Str(s)),
                Some('\\') => match self.advance() {
                    Some('"') => s.push('"'),
                    Some('\\') => s.push('\\'),
                    Some('n') => s.push('\n'),
                    Some(other) => {
                        return Err(SourceError::new(
                            self.line,
                            self.col.saturating_sub(1),
                            format!("unknown escape sequence '\\{}'", other),
                        ));
                    }
                    None => break,
                },
                Some(c) => s.push(c),
                None => break,
            }
        }
        Err(SourceError::new(
            start_line,
            start_col,
            "unterminated string literal",
        ))
    }
}
