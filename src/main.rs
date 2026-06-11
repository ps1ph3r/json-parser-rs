#[derive(Debug, Clone, PartialEq)]
pub enum LexError {
    UnexpectedChar(char),
    UnexpectedEOF(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedEOF,
    UnexpectedToken(String),
    ExpectedToken { expected: String, found: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    True,
    False,
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
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
            Some(c) => return Err(LexError::UnexpectedChar(c as char)),
        }
    }
    Ok(tokens)
}

fn main() {
    let input = "true  false null";
    println!("testing lexer: {}", input);
    match tokenize(input) {
        Ok(tokens) => println!("Tokens: {:?}", tokens),
        Err(e) => println!("Error: {:?}", e),
    }
}
