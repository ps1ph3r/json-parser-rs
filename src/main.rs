#[derive(Debug, Clone, PartialEq)]
pub enum LexError {
    UnexpectedChar(char),
    UnexpectedEOF(String),
    UnterminatedString,
    InvalidEscape(String),
    InvalidUnicode(u32),
    InvalidHexDigit(char),
    InvalidNumber(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedEOF,
    UnexpectedToken(String),
    ExpectedToken { expected: String, found: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    StringToken(String),
    NumberToken(f64),
    True,
    False,
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Str(String),
    Number(f64),
    Bool(bool),
    Null,
}

pub struct Lexer {
    input: Vec<u8>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.as_bytes().to_vec(),
            pos: 0,
        }
    }

    fn current(&self) -> Option<u8> {
        if self.pos < self.input.len() {
            Some(self.input[self.pos])
        } else {
            None
        }
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn read_keyword(&mut self, keyword: &str) -> Result<(), LexError> {
        for expected in keyword.as_bytes() {
            match self.current() {
                Some(c) if c == *expected => self.advance(),
                Some(c) => return Err(LexError::UnexpectedChar(c as char)),
                None => {
                    return Err(LexError::UnexpectedEOF(format!(
                        "reading keyword '{}'",
                        keyword
                    )));
                }
            }
        }
        Ok(())
    }

    fn read_string(&mut self) -> Result<String, LexError> {
        self.advance(); // Skip starting quote
        let mut result = String::new();
        loop {
            match self.current() {
                None => return Err(LexError::UnterminatedString),
                Some(b'"') => {
                    self.advance();
                    return Ok(result);
                }
                Some(b'\\') => {
                    self.advance();
                    match self.current() {
                        Some(b'"') => {
                            result.push('"');
                            self.advance();
                        }
                        Some(b'\\') => {
                            result.push('\\');
                            self.advance();
                        }
                        Some(b'/') => {
                            result.push('/');
                            self.advance();
                        }
                        Some(b'n') => {
                            result.push('\n');
                            self.advance();
                        }
                        Some(b't') => {
                            result.push('\t');
                            self.advance();
                        }
                        Some(b'r') => {
                            result.push('\r');
                            self.advance();
                        }
                        Some(b'b') => {
                            result.push('\x08');
                            self.advance();
                        }
                        Some(b'f') => {
                            result.push('\x0C');
                            self.advance();
                        }
                        Some(b'u') => {
                            self.advance();
                            let cp = self.read_unicode_escape()?;
                            let ch =
                                char::from_u32(cp).ok_or_else(|| LexError::InvalidUnicode(cp))?;
                            result.push(ch);
                        }
                        Some(c) => return Err(LexError::InvalidEscape(format!("\\{}", c as char))),
                        None => {
                            return Err(LexError::UnexpectedEOF(
                                "Unterminated escape sequence".to_string(),
                            ));
                        }
                    }
                }
                Some(c) => {
                    result.push(c as char);
                    self.advance();
                }
            }
        }
    }

    fn read_unicode_escape(&mut self) -> Result<u32, LexError> {
        let mut value: u32 = 0;
        for _ in 0..4 {
            match self.current() {
                Some(c) => {
                    let digit = match c {
                        b'0'..=b'9' => (c - b'0') as u32,
                        b'a'..=b'f' => (c - b'a' + 10) as u32,
                        b'A'..=b'F' => (c - b'A' + 10) as u32,
                        _ => return Err(LexError::InvalidHexDigit(c as char)),
                    };
                    value = value * 16 + digit;
                    self.advance();
                }
                None => {
                    return Err(LexError::UnexpectedEOF(
                        "Unterminated unicode escape".to_string(),
                    ));
                }
            }
        }
        Ok(value)
    }

    fn read_number(&mut self) -> Result<f64, LexError> {
        let mut s = String::new();
        if let Some(b'-') = self.current() {
            s.push('-');
            self.advance();
        }
        loop {
            match self.current() {
                Some(c @ b'0'..=b'9') => {
                    s.push(c as char);
                    self.advance();
                }
                _ => break,
            }
        }
        if let Some(b'.') = self.current() {
            s.push('.');
            self.advance();
            loop {
                match self.current() {
                    Some(c @ b'0'..=b'9') => {
                        s.push(c as char);
                        self.advance();
                    }
                    _ => break,
                }
            }
        }
        s.parse::<f64>().map_err(|_| LexError::InvalidNumber(s))
    }
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, LexError> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();
    loop {
        match lexer.current() {
            None => break,
            Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r') => {
                lexer.advance();
            }
            Some(b't') => {
                lexer.read_keyword("true")?;
                tokens.push(Token::True);
            }
            Some(b'f') => {
                lexer.read_keyword("false")?;
                tokens.push(Token::False);
            }
            Some(b'n') => {
                lexer.read_keyword("null")?;
                tokens.push(Token::Null);
            }
            Some(b'"') => {
                let s = lexer.read_string()?;
                tokens.push(Token::StringToken(s));
            }
            Some(b'-') | Some(b'0'..=b'9') => {
                let n = lexer.read_number()?;
                tokens.push(Token::NumberToken(n));
            }
            Some(c) => return Err(LexError::UnexpectedChar(c as char)),
        }
    }
    Ok(tokens)
}

fn main() {
    let input = r#"true false "hello\nworld" -42.5 100 null"#;
    println!("testing lexer: {}", input);
    match tokenize(input) {
        Ok(tokens) => println!("Tokens: {:?}", tokens),
        Err(e) => println!("Error: {:?}", e),
    }
}
