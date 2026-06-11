#[derive(Debug, Clone, PartialEq)]
pub enum LexError {
    UnexpectedChar(char),
    UnexpectedEOF(String),
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
    NumberToken(f64),
    True,
    False,
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
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
    let input = "true  false -42.5 100 null";
    println!("testing lexer: {}", input);
    match tokenize(input) {
        Ok(tokens) => println!("Tokens: {:?}", tokens),
        Err(e) => println!("Error: {:?}", e),
    }
}
