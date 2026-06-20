// errors
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

// ast / value
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Colon,
    Comma,
    StringToken(String),
    NumberToken(f64),
    True,
    False,
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Object(Vec<(String, JsonValue)>),
    Array(Vec<JsonValue>),
    Str(String),
    Number(f64),
    Bool(bool),
    Null,
}

// lexer
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
            Some(b'{') => {
                tokens.push(Token::LeftBrace);
                lexer.advance();
            }
            Some(b'}') => {
                tokens.push(Token::RightBrace);
                lexer.advance();
            }
            Some(b'[') => {
                tokens.push(Token::LeftBracket);
                lexer.advance();
            }
            Some(b']') => {
                tokens.push(Token::RightBracket);
                lexer.advance();
            }
            Some(b':') => {
                tokens.push(Token::Colon);
                lexer.advance();
            }
            Some(b',') => {
                tokens.push(Token::Comma);
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

// parser
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn current(&self) -> Result<&Token, ParseError> {
        if self.pos < self.tokens.len() {
            Ok(&self.tokens[self.pos])
        } else {
            Err(ParseError::UnexpectedEOF)
        }
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn expect(&mut self, description: &str) -> Result<&Token, ParseError> {
        if self.pos < self.tokens.len() {
            let token = &self.tokens[self.pos];
            self.pos += 1;
            Ok(token)
        } else {
            Err(ParseError::ExpectedToken {
                expected: description.to_string(),
                found: "EOF".to_string(),
            })
        }
    }

    pub fn parse_value(&mut self) -> Result<JsonValue, ParseError> {
        match self.current()? {
            Token::LeftBrace => self.parse_object(),
            Token::LeftBracket => self.parse_array(),
            Token::True => {
                self.advance();
                Ok(JsonValue::Bool(true))
            }
            Token::False => {
                self.advance();
                Ok(JsonValue::Bool(false))
            }
            Token::Null => {
                self.advance();
                Ok(JsonValue::Null)
            }
            Token::NumberToken(n) => {
                let v = *n;
                self.advance();
                Ok(JsonValue::Number(v))
            }
            Token::StringToken(s) => {
                let v = s.clone();
                self.advance();
                Ok(JsonValue::Str(v))
            }
            Token::RightBrace => Err(ParseError::UnexpectedToken("}".to_string())),
            Token::RightBracket => Err(ParseError::UnexpectedToken("]".to_string())),
            Token::Colon => Err(ParseError::UnexpectedToken(":".to_string())),
            Token::Comma => Err(ParseError::UnexpectedToken(",".to_string())),
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, ParseError> {
        self.advance(); // step past LeftBrace
        let mut pairs: Vec<(String, JsonValue)> = Vec::new();

        if let Token::RightBrace = self.current()? {
            self.advance();
            return Ok(JsonValue::Object(pairs));
        }

        loop {
            let key = match self.expect("object key")? {
                Token::StringToken(s) => s.clone(),
                t => {
                    return Err(ParseError::ExpectedToken {
                        expected: "String Key".to_string(),
                        found: format!("{:?}", t),
                    });
                }
            };

            match self.expect("colon")? {
                Token::Colon => {}
                t => {
                    return Err(ParseError::ExpectedToken {
                        expected: ":".to_string(),
                        found: format!("{:?}", t),
                    });
                }
            }

            let value = self.parse_value()?;
            pairs.push((key, value));

            match self.current()? {
                Token::Comma => {
                    self.advance();
                }
                Token::RightBrace => {
                    self.advance();
                    break;
                }
                t => return Err(ParseError::UnexpectedToken(format!("{:?}", t))),
            }
        }

        Ok(JsonValue::Object(pairs))
    }

    fn parse_array(&mut self) -> Result<JsonValue, ParseError> {
        self.advance(); // step past LeftBracket
        let mut elements: Vec<JsonValue> = Vec::new();

        if let Token::RightBracket = self.current()? {
            self.advance();
            return Ok(JsonValue::Array(elements));
        }

        loop {
            let element = self.parse_value()?;
            elements.push(element);

            match self.current()? {
                Token::Comma => {
                    self.advance();
                }
                Token::RightBracket => {
                    self.advance();
                    break;
                }
                t => return Err(ParseError::UnexpectedToken(format!("{:?}", t))),
            }
        }

        Ok(JsonValue::Array(elements))
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<JsonValue, ParseError> {
    let mut parser = Parser::new(tokens);
    let value = parser.parse_value()?;

    if parser.pos < parser.tokens.len() {
        let trailing_token = &parser.tokens[parser.pos];
        return Err(ParseError::UnexpectedToken(format!(
            "Trailing tokens found after valid JSON value: {:?}",
            trailing_token
        )));
    }

    Ok(value)
}

// display
fn display(value: &JsonValue) -> String {
    match value {
        JsonValue::Null => String::from("null"),
        JsonValue::Bool(true) => String::from("true"),
        JsonValue::Bool(false) => String::from("false"),
        JsonValue::Number(n) => format!("{}", n),
        JsonValue::Str(s) => format!("\"{}\"", s),
        JsonValue::Array(elements) => {
            let mut result = String::from("[");
            let mut first = true;
            for element in elements {
                if !first {
                    result.push_str(", ");
                }
                result.push_str(&display(element));
                first = false;
            }
            result.push(']');
            result
        }
        JsonValue::Object(pairs) => {
            let mut result = String::from("{");
            let mut first = true;
            for (key, val) in pairs {
                if !first {
                    result.push_str(", ");
                }
                result.push_str(&format!("\"{}\": {}", key, display(val)));
                first = false;
            }
            result.push('}');
            result
        }
    }
}

// main
fn main() {
    let tests = vec![
        r#"null"#,
        r#"42"#,
        r#"[1, 2, 3]"#,
        r#"{"name": "alice", "age": 30}"#,
        r#"{"invalid_json": }"#,
    ];

    for input in &tests {
        println!("Input:  {}", input);
        match tokenize(input) {
            Ok(tokens) => {
                println!("Tokens: {:?}", tokens);
                match parse(tokens) {
                    Ok(value) => println!("Output: {}", display(&value)),
                    Err(e) => println!("Parser Error: {:?}", e),
                }
            }
            Err(e) => println!("Lexer Error: {:?}", e),
        }
        println!("---");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Lexer Tests

    #[test]
    fn test_tokenize_primitives() {
        let input = "true false null";
        let expected = vec![Token::True, Token::False, Token::Null];
        assert_eq!(tokenize(input).unwrap(), expected);
    }

    #[test]
    fn test_tokenize_numbers() {
        let input = "0 42 -17 3.1415 -0.001";
        let expected = vec![
            Token::NumberToken(0.0),
            Token::NumberToken(42.0),
            Token::NumberToken(-17.0),
            Token::NumberToken(3.1415),
            Token::NumberToken(-0.001),
        ];
        assert_eq!(tokenize(input).unwrap(), expected);
    }

    #[test]
    fn test_tokenize_string_escapes() {
        let input = r#""hello\nworld" "tab\t" "quote\"" "slash\\" "unicode \u0041""#;
        let tokens = tokenize(input).unwrap();

        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], Token::StringToken("hello\nworld".to_string()));
        assert_eq!(tokens[1], Token::StringToken("tab\t".to_string()));
        assert_eq!(tokens[2], Token::StringToken("quote\"".to_string()));
        assert_eq!(tokens[3], Token::StringToken("slash\\".to_string()));
        assert_eq!(tokens[4], Token::StringToken("unicode A".to_string()));
    }

    #[test]
    fn test_tokenize_structural_symbols() {
        let input = "{ } [ ] : ,";
        let expected = vec![
            Token::LeftBrace,
            Token::RightBrace,
            Token::LeftBracket,
            Token::RightBracket,
            Token::Colon,
            Token::Comma,
        ];
        assert_eq!(tokenize(input).unwrap(), expected);
    }

    #[test]
    fn test_tokenize_lexer_errors() {
        // Unexpected character
        assert_eq!(tokenize("@"), Err(LexError::UnexpectedChar('@')));

        // Unterminated string
        assert_eq!(
            tokenize(r#""unclosed string"#),
            Err(LexError::UnterminatedString)
        );

        // Invalid escape sequence
        assert_eq!(
            tokenize(r#""bad escape \x""#),
            Err(LexError::InvalidEscape("\\x".to_string()))
        );

        // Incomplete keyword EOF
        assert_eq!(
            tokenize("tru"),
            Err(LexError::UnexpectedEOF(
                "reading keyword 'true'".to_string()
            ))
        );

        // Invalid Hex Digit in Unicode escape
        assert_eq!(tokenize(r#""\u000G""#), Err(LexError::InvalidHexDigit('G')));
    }

    // Parser Tests

    #[test]
    fn test_parse_primitives() {
        assert_eq!(parse(vec![Token::Null]).unwrap(), JsonValue::Null);
        assert_eq!(parse(vec![Token::True]).unwrap(), JsonValue::Bool(true));
        assert_eq!(parse(vec![Token::False]).unwrap(), JsonValue::Bool(false));
        assert_eq!(
            parse(vec![Token::NumberToken(123.45)]).unwrap(),
            JsonValue::Number(123.45)
        );
        assert_eq!(
            parse(vec![Token::StringToken("test".to_string())]).unwrap(),
            JsonValue::Str("test".to_string())
        );
    }

    #[test]
    fn test_parse_arrays() {
        // Empty array
        let tokens = vec![Token::LeftBracket, Token::RightBracket];
        assert_eq!(parse(tokens).unwrap(), JsonValue::Array(vec![]));

        // Mixed array
        let tokens = vec![
            Token::LeftBracket,
            Token::NumberToken(1.0),
            Token::Comma,
            Token::StringToken("a".to_string()),
            Token::Comma,
            Token::True,
            Token::RightBracket,
        ];
        let expected = JsonValue::Array(vec![
            JsonValue::Number(1.0),
            JsonValue::Str("a".to_string()),
            JsonValue::Bool(true),
        ]);
        assert_eq!(parse(tokens).unwrap(), expected);
    }

    #[test]
    fn test_parse_objects() {
        // Empty object
        let tokens = vec![Token::LeftBrace, Token::RightBrace];
        assert_eq!(parse(tokens).unwrap(), JsonValue::Object(vec![]));

        // Object with values
        let tokens = vec![
            Token::LeftBrace,
            Token::StringToken("key".to_string()),
            Token::Colon,
            Token::NumberToken(42.0),
            Token::RightBrace,
        ];
        let expected = JsonValue::Object(vec![("key".to_string(), JsonValue::Number(42.0))]);
        assert_eq!(parse(tokens).unwrap(), expected);
    }

    #[test]
    fn test_parse_errors() {
        // Trailing tokens
        let tokens = vec![Token::NumberToken(1.0), Token::NumberToken(2.0)];
        assert!(matches!(parse(tokens), Err(ParseError::UnexpectedToken(_))));

        // Non-string object key
        let tokens = vec![
            Token::LeftBrace,
            Token::NumberToken(123.0),
            Token::Colon,
            Token::True,
            Token::RightBrace,
        ];
        assert!(matches!(
            parse(tokens),
            Err(ParseError::ExpectedToken { .. })
        ));

        // Missing colon before EOF
        let tokens = vec![Token::LeftBrace, Token::StringToken("key".to_string())];
        assert_eq!(
            parse(tokens),
            Err(ParseError::ExpectedToken {
                expected: "colon".to_string(),
                found: "EOF".to_string(),
            })
        );
    }
    // Pretty Printing Tests

    #[test]
    fn test_display() {
        let ast = JsonValue::Object(vec![
            ("name".to_string(), JsonValue::Str("Alice".to_string())),
            (
                "skills".to_string(),
                JsonValue::Array(vec![
                    JsonValue::Str("Rust".to_string()),
                    JsonValue::Str("JSON".to_string()),
                ]),
            ),
            ("active".to_string(), JsonValue::Bool(true)),
        ]);

        let expected = r#"{"name": "Alice", "skills": ["Rust", "JSON"], "active": true}"#;
        assert_eq!(display(&ast), expected);
    }

    // End-to-End Pipeline Tests

    #[test]
    fn test_e2e_valid_json() {
        let input = r#"{
            "title": "Parser",
            "count": 10,
            "items": [1, false, null],
            "nested": { "ok": true }
        }"#;

        let tokens = tokenize(input).expect("Tokenization should succeed");
        let parsed = parse(tokens).expect("Parsing should succeed");

        assert_eq!(
            parsed,
            JsonValue::Object(vec![
                ("title".to_string(), JsonValue::Str("Parser".to_string())),
                ("count".to_string(), JsonValue::Number(10.0)),
                (
                    "items".to_string(),
                    JsonValue::Array(vec![
                        JsonValue::Number(1.0),
                        JsonValue::Bool(false),
                        JsonValue::Null,
                    ])
                ),
                (
                    "nested".to_string(),
                    JsonValue::Object(vec![("ok".to_string(), JsonValue::Bool(true))])
                )
            ])
        );
    }
}
