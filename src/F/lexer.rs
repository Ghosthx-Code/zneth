#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Static,
    Signed,
    Unsigned,
    Fn,
    Damtic,
    Void,
    I32,
    I64,
    I128,
    F32,
    F64,
    I8,
    I1,
    Str,
    Char,
    Printf,
    String,
    Num,
    Float,
    Ret,
    Module,
    SemiColoun,
    LeftCurl,
    RightCurl, // Fixed: This now maps correctly!
    LeftParen,
    RightParen,
    Question,
    Id,
    Equal,
    Mut,
    Bang,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub typ: TokenKind,
    pub value: String,
    pub line: i32,
}

impl Token {
    pub fn new(typ: TokenKind, value: String, line: i32) -> Self {
        Self { typ, value, line }
    }
}

pub struct Lexer {
    pub source: String,
    pub pos: usize,
    pub current_line: i32,
    pub start: usize,
}

impl Lexer {
    pub fn new(source: String) -> Self {
        Self {
            source,
            pos: 0,
            current_line: 1,
            start: 0,
        }
    }

    pub fn is_at_end(&self) -> bool {
        self.pos >= self.source.len()
    }

    pub fn adv(&mut self) -> Option<char> {
        if self.is_at_end() {
            return None;
        }
        let bytes = self.source.as_bytes();
        let b = bytes[self.pos];
        if b < 128 {
            self.pos += 1;
            Some(b as char)
        } else {
            let current_char = self.source[self.pos..].chars().next()?;
            self.pos += current_char.len_utf8();
            Some(current_char)
        }
    }

    pub fn peek(&self) -> char {
        let bytes = self.source.as_bytes();
        if self.pos >= bytes.len() {
            return '\0';
        }
        let b = bytes[self.pos];
        if b < 128 {
            b as char
        } else {
            self.source[self.pos..].chars().next().unwrap_or('\0')
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while !self.is_at_end() {
            self.start = self.pos;
            if let Some(token) = self.match_tokens() {
                tokens.push(token);
            }
        }
        tokens.push(Token::new(
            TokenKind::Eof,
            "".to_string(),
            self.current_line,
        ));
        tokens
    }

    #[inline(always)]
    pub fn match_tokens(&mut self) -> Option<Token> {
        let c: char = self.adv()?;

        // FIX: Handle and skip whitespace safely
        if c.is_whitespace() {
            if c == '\n' {
                self.current_line += 1;
            }
            return None;
        }

        let kind = match c as u8 {
            b'(' => TokenKind::LeftParen,
            b')' => TokenKind::RightParen,
            b'{' => TokenKind::LeftCurl,
            b'}' => TokenKind::RightCurl, // FIX: Typo corrected from RightParen
            b';' => TokenKind::SemiColoun,
            b'?' => TokenKind::Question,
            b'!' => TokenKind::Bang,
            b'=' => TokenKind::Equal,
            b'"' => return self.handle_str(),
            b'\'' => {
                self.adv();
                let c = self.peek();
                if self.peek() != '\'' {
                    panic!();
                }
                self.adv();
                let lexme = self.source[self.start..self.pos].to_string();
                return Some(Token::new(
                    TokenKind::Char,
                    self.source[self.start..self.pos].to_string(),
                    self.current_line,
                ));
            }
            _ => {
                // FIX: Differentiate numbers from words/identifiers
                if c.is_ascii_digit() {
                    return self.handle_num_or_float();
                } else if c.is_alphabetic() || c == '_' {
                    return self.handle_id();
                } else {
                    eprintln!(
                        "Lexer Error: Unknown symbol '{}' found on line {}.",
                        c, self.current_line
                    );
                    return None;
                }
            }
        };

        Some(Token::new(
            kind,
            self.source[self.start..self.pos].to_string(),
            self.current_line,
        ))
    }

    fn handle_str(&mut self) -> Option<Token> {
        while self.peek() != '\"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.current_line += 1;
            }
            self.adv();
        }
        if self.is_at_end() {
            eprintln!(
                "Lexer Error: Unterminated string literal on line {}.",
                self.current_line
            );
            return None;
        }
        self.adv(); // Consume closing quote
        let lexme = &self.source[self.start + 1..self.pos - 1];
        Some(Token::new(
            TokenKind::String,
            lexme.to_string(),
            self.current_line,
        ))
    }

    fn handle_id(&mut self) -> Option<Token> {
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.adv();
        }
        let text = &self.source[self.start..self.pos];
        let kind = match text.as_bytes() {
            b"printf" => TokenKind::Printf,
            b"static" => TokenKind::Static,
            b"damtic" => TokenKind::Damtic,
            b"fn" => TokenKind::Fn,
            b"signed" => TokenKind::Signed,
            b"unsigned" => TokenKind::Unsigned,
            b"mut" => TokenKind::Mut,
            b"ret" => TokenKind::Ret,
            b"i32" => TokenKind::I32,
            b"i64" => TokenKind::I64,
            b"i128" => TokenKind::I128,
            b"f32" => TokenKind::F32,
            b"f64" => TokenKind::F64,
            b"str" => TokenKind::Str,
            b"i8" => TokenKind::I8,
            b"i1" => TokenKind::I1,
            b"void" => TokenKind::Void,
            b"module" => TokenKind::Module,
            _ => TokenKind::Id,
        };
        Some(Token::new(kind, text.to_string(), self.current_line))
    }

    pub fn peek_next(&self) -> char {
        let bytes = self.source.as_bytes();
        if self.pos >= bytes.len() {
            return '\0';
        }
        let b = bytes[self.pos];
        let current_char_len = if b < 128 {
            1
        } else {
            self.source[self.pos..]
                .chars()
                .next()
                .unwrap_or('\0')
                .len_utf8()
        };
        if self.pos + current_char_len >= bytes.len() {
            return '\0';
        }
        let next_b = bytes[self.pos + current_char_len];
        if next_b < 128 {
            next_b as char
        } else {
            self.source[self.pos + current_char_len..]
                .chars()
                .next()
                .unwrap_or('\0')
        }
    }

    // FIX: Extracted numeric literals helper
    fn handle_num_or_float(&mut self) -> Option<Token> {
        while self.peek().is_ascii_digit() {
            self.adv();
        }
        let mut is_float = false;
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            is_float = true;
            self.adv();
            while self.peek().is_ascii_digit() {
                self.adv();
            }
        }
        let kind = if is_float {
            TokenKind::Float
        } else {
            TokenKind::Num
        };
        let lexeme = &self.source[self.start..self.pos];
        Some(Token::new(kind, lexeme.to_string(), self.current_line))
    }
}
