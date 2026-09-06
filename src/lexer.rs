use crate::token::{Token, TokenKind};

#[derive(Debug, Clone, PartialEq)]
pub struct LexError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

pub struct Lexer<'a> {
    source: &'a [u8],
    current: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source: source.as_bytes(), current: 0, line: 1, column: 1 }
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        while !self.is_at_end() {
            self.skip_whitespace_and_comments();
            if self.is_at_end() { break; }
            tokens.push(self.next_token()?);
        }
        tokens.push(Token::new(TokenKind::Eof, "", self.line, self.column));
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, LexError> {
        let line = self.line;
        let column = self.column;
        let start = self.current;
        let byte = self.advance();
        let kind = match byte {
            b'(' => TokenKind::LeftParen,
            b')' => TokenKind::RightParen,
            b'{' => TokenKind::LeftBrace,
            b'}' => TokenKind::RightBrace,
            b',' => TokenKind::Comma,
            b';' => TokenKind::Semicolon,
            b':' => TokenKind::Colon,
            b'+' => TokenKind::Plus,
            b'-' => TokenKind::Minus,
            b'*' => TokenKind::Star,
            b'%' => TokenKind::Percent,
            b'/' => TokenKind::Slash,
            b'=' if self.match_byte(b'=') => TokenKind::EqualEqual,
            b'=' => TokenKind::Equal,
            b'!' if self.match_byte(b'=') => TokenKind::BangEqual,
            b'!' => TokenKind::Bang,
            b'<' if self.match_byte(b'=') => TokenKind::LessEqual,
            b'<' => TokenKind::Less,
            b'>' if self.match_byte(b'=') => TokenKind::GreaterEqual,
            b'>' => TokenKind::Greater,
            b'&' if self.match_byte(b'&') => TokenKind::AndAnd,
            b'|' if self.match_byte(b'|') => TokenKind::OrOr,
            b'"' => return self.string_token(start, line, column),
            b'0'..=b'9' => return self.number_token(start, line, column),
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => return self.identifier_token(start, line, column),
            _ => return Err(LexError { message: format!("unexpected character '{}'.", byte as char), line, column }),
        };
        Ok(Token::new(kind, String::from_utf8_lossy(&self.source[start..self.current]), line, column))
    }

    fn identifier_token(&mut self, start: usize, line: usize, column: usize) -> Result<Token, LexError> {
        while self.peek().is_ascii_alphanumeric() || self.peek() == b'_' { self.advance(); }
        let lexeme = String::from_utf8_lossy(&self.source[start..self.current]).to_string();
        let kind = match lexeme.as_str() {
            "let" => TokenKind::Let,
            "fn" => TokenKind::Fn,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "return" => TokenKind::Return,
            "print" => TokenKind::Print,
            _ => TokenKind::Identifier(lexeme.clone()),
        };
        Ok(Token::new(kind, lexeme, line, column))
    }

    fn number_token(&mut self, start: usize, line: usize, column: usize) -> Result<Token, LexError> {
        while self.peek().is_ascii_digit() { self.advance(); }
        if self.peek() == b'.' && self.peek_next().is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() { self.advance(); }
        }
        let lexeme = String::from_utf8_lossy(&self.source[start..self.current]).to_string();
        let value = lexeme.parse::<f64>().map_err(|_| LexError { message: format!("invalid number '{}'.", lexeme), line, column })?;
        Ok(Token::new(TokenKind::Number(value), lexeme, line, column))
    }

    fn string_token(&mut self, start: usize, line: usize, column: usize) -> Result<Token, LexError> {
        let content_start = self.current;
        while !self.is_at_end() && self.peek() != b'"' { self.advance(); }
        if self.is_at_end() { return Err(LexError { message: "unterminated string literal.".into(), line, column }); }
        let content = String::from_utf8_lossy(&self.source[content_start..self.current]).to_string();
        self.advance();
        Ok(Token::new(TokenKind::String(content), String::from_utf8_lossy(&self.source[start..self.current]), line, column))
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.peek() {
                b' ' | b'\r' | b'\t' | b'\n' => { self.advance(); }
                b'/' if self.peek_next() == b'/' => { while !self.is_at_end() && self.peek() != b'\n' { self.advance(); } }
                _ => break,
            }
        }
    }

    fn advance(&mut self) -> u8 {
        let byte = self.source[self.current];
        self.current += 1;
        if byte == b'\n' { self.line += 1; self.column = 1; } else { self.column += 1; }
        byte
    }

    fn match_byte(&mut self, expected: u8) -> bool {
        if self.peek() != expected { return false; }
        self.advance();
        true
    }
    fn peek(&self) -> u8 { self.source.get(self.current).copied().unwrap_or(0) }
    fn peek_next(&self) -> u8 { self.source.get(self.current + 1).copied().unwrap_or(0) }
    fn is_at_end(&self) -> bool { self.current >= self.source.len() }
}
