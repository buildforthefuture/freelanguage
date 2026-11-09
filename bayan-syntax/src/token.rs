use crate::span::{Loc, Span};

/// A single token with its span and the source text it covers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub text: String,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span, text: impl Into<String>) -> Self {
        Token { kind, span, text: text.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // Keywords
    Fn, Extern, Export, Return, If, Else, Loop, Break, Continue, Var, Let,
    Unsafe, Asm, True, False, Trap, Goto, Module, Import, Struct, Align,
    Cc, Pure, NoInline, Inline, Cold, Volatile, Default, Case, Global, Switch,

    // Identifiers & literals
    Ident(String),
    IntLit(u64, IntSuffix),
    FloatLit(String, FloatSuffix),
    StringLit(String),
    CharLit(char),

    // Punctuation / operators
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Comma, Colon, Semicolon, Arrow, FatArrow, Equals, Dot,
    Plus, Minus, Star, Slash, Percent,
    Lt, Gt,
    Eq, Ne, LtS, LtU, LeS, LeU, GtS, GtU, GeS, GeU,
    And, Or, Not,

    // Special
    Null,
    Eof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntSuffix {
    None, I8, I16, I32, I64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatSuffix {
    None, F32, F64,
}

impl TokenKind {
    pub fn is_keyword_kind(&self) -> bool {
        matches!(
            self,
            TokenKind::Fn | TokenKind::Extern | TokenKind::Export | TokenKind::Return
                | TokenKind::If | TokenKind::Else | TokenKind::Loop | TokenKind::Break
                | TokenKind::Continue | TokenKind::Var | TokenKind::Let | TokenKind::Unsafe
                | TokenKind::Asm | TokenKind::True | TokenKind::False | TokenKind::Trap
                | TokenKind::Goto | TokenKind::Module | TokenKind::Import
                | TokenKind::Struct | TokenKind::Align | TokenKind::Cc | TokenKind::Pure
                | TokenKind::NoInline | TokenKind::Inline | TokenKind::Cold | TokenKind::Volatile
                | TokenKind::Default | TokenKind::Case | TokenKind::Global | TokenKind::Switch
        )
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::Extern => write!(f, "extern"),
            TokenKind::Export => write!(f, "export"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::Loop => write!(f, "loop"),
            TokenKind::Break => write!(f, "break"),
            TokenKind::Continue => write!(f, "continue"),
            TokenKind::Var => write!(f, "var"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::Unsafe => write!(f, "unsafe"),
            TokenKind::Asm => write!(f, "asm"),
            TokenKind::True => write!(f, "true"),
            TokenKind::False => write!(f, "false"),
            TokenKind::Trap => write!(f, "trap"),
            TokenKind::Goto => write!(f, "goto"),
            TokenKind::Module => write!(f, "module"),
            TokenKind::Import => write!(f, "import"),
            TokenKind::Struct => write!(f, "struct"),
            TokenKind::Align => write!(f, "align"),
            TokenKind::Cc => write!(f, "cc"),
            TokenKind::Pure => write!(f, "pure"),
            TokenKind::NoInline => write!(f, "noinline"),
            TokenKind::Inline => write!(f, "inline"),
            TokenKind::Cold => write!(f, "cold"),
            TokenKind::Volatile => write!(f, "volatile"),
            TokenKind::Default => write!(f, "default"),
            TokenKind::Case => write!(f, "case"),
            TokenKind::Global => write!(f, "global"),
            TokenKind::Switch => write!(f, "switch"),
            TokenKind::Ident(s) => write!(f, "ident({})", s),
            TokenKind::IntLit(_, s) => write!(f, "int({:?})", s),
            TokenKind::FloatLit(_, s) => write!(f, "float({:?})", s),
            TokenKind::StringLit(s) => write!(f, "string({:?})", s),
            TokenKind::CharLit(c) => write!(f, "char({:?})", c),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::LBracket => write!(f, "["),
            TokenKind::RBracket => write!(f, "]"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::FatArrow => write!(f, "=>"),
            TokenKind::Equals => write!(f, "="),
            TokenKind::Dot => write!(f, "."),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::Lt => write!(f, "<"),
            TokenKind::Gt => write!(f, ">"),
            TokenKind::Eq => write!(f, "=="),
            TokenKind::Ne => write!(f, "!="),
            TokenKind::LtS => write!(f, "lt.s"),
            TokenKind::LtU => write!(f, "lt.u"),
            TokenKind::LeS => write!(f, "le.s"),
            TokenKind::LeU => write!(f, "le.u"),
            TokenKind::GtS => write!(f, "gt.s"),
            TokenKind::GtU => write!(f, "gt.u"),
            TokenKind::GeS => write!(f, "ge.s"),
            TokenKind::GeU => write!(f, "ge.u"),
            TokenKind::And => write!(f, "and"),
            TokenKind::Or => write!(f, "or"),
            TokenKind::Not => write!(f, "not"),
            TokenKind::Null => write!(f, "null"),
            TokenKind::Eof => write!(f, "eof"),
        }
    }
}

use std::fmt;

/// The lexer produces tokens from source text.
pub struct Lexer<'src> {
    src: &'src str,
    bytes: &'src [u8],
    pos: usize,
    line: usize,
    col: usize,
}

impl<'src> Lexer<'src> {
    pub fn new(src: &'src str) -> Self {
        Lexer {
            src,
            bytes: src.as_bytes(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token()?;
            let clone = tok.clone();
            tokens.push(tok);
            if matches!(clone.kind, TokenKind::Eof) {
                break;
            }
        }
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, LexError> {
        self.skip_ws_and_comments();
        if self.pos >= self.bytes.len() {
            return Ok(Token::new(TokenKind::Eof, Span::none(), ""));
        }
        let start = self.pos();
        let b = self.bytes[self.pos];

        if b.is_ascii_digit() {
            return self.scan_number(start);
        }
        if b.is_ascii_alphabetic() || b == b'_' {
            return self.scan_ident_or_keyword(start);
        }
        if b == b'"' {
            return self.scan_string(start);
        }
        if b == b'\'' {
            return self.scan_char(start);
        }

        // Multi-char punctuation / operators
        if let Some(tok) = self.scan_punct(start)? {
            return Ok(tok);
        }

        Err(LexError::InvalidChar(self.pos().1, b as char))
    }

    fn pos(&self) -> (Loc, usize) {
        (Loc { line: self.line, col: self.col }, self.pos)
    }

    fn span_from(&self, start_loc: Loc, start_byte: usize) -> Span {
        Span {
            start: start_loc,
            end: Loc { line: self.line, col: self.col },
            byte_start: start_byte,
            byte_end: self.pos,
        }
    }

    fn skip_ws_and_comments(&mut self) {
        loop {
            while self.pos < self.bytes.len() {
                match self.bytes[self.pos] {
                    b' ' | b'\t' | b'\r' => { self.pos += 1; self.col += 1; }
                    b'\n' => { self.pos += 1; self.line += 1; self.col = 1; }
                    b'/' if self.pos + 1 < self.bytes.len() && self.bytes[self.pos + 1] == b'/' => {
                        // line comment
                        while self.pos < self.bytes.len() && self.bytes[self.pos] != b'\n' {
                            self.pos += 1;
                            self.col += 1;
                        }
                    }
                    b'/' if self.pos + 1 < self.bytes.len() && self.bytes[self.pos + 1] == b'*' => {
                        // block comment (nesting)
                        self.pos += 2;
                        self.col += 2;
                        let mut depth: usize = 1;
                        while depth > 0 && self.pos < self.bytes.len() {
                            if self.bytes[self.pos] == b'/' && self.pos + 1 < self.bytes.len() && self.bytes[self.pos + 1] == b'*' {
                                depth += 1;
                                self.pos += 2;
                                self.col += 2;
                            } else if self.bytes[self.pos] == b'*' && self.pos + 1 < self.bytes.len() && self.bytes[self.pos + 1] == b'/' {
                                depth -= 1;
                                self.pos += 2;
                                self.col += 2;
                            } else {
                                if self.bytes[self.pos] == b'\n' { self.line += 1; self.col = 1; } else { self.col += 1; }
                                self.pos += 1;
                            }
                        }
                        if depth > 0 {
                            // unterminated - handled by caller? We just stop at EOF.
                        }
                    }
                    _ => break,
                }
            }
            // re-check: if we just consumed a comment, there may be more ws
            let b = if self.pos < self.bytes.len() { self.bytes[self.pos] } else { break };
            if !(b.is_ascii_whitespace() || b == b'/') {
                break;
            }
        }
    }

    fn scan_ident_or_keyword(&mut self, start: (Loc, usize)) -> Result<Token, LexError> {
        let start_byte = start.1;
        let start_loc = start.0;
        while self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            if b.is_ascii_alphanumeric() || b == b'_' {
                self.pos += 1;
                self.col += 1;
            } else {
                break;
            }
        }
        let text = &self.src[start_byte..self.pos];
        let kind = match text {
            "fn" => TokenKind::Fn,
            "extern" => TokenKind::Extern,
            "export" => TokenKind::Export,
            "return" => TokenKind::Return,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "loop" => TokenKind::Loop,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "var" => TokenKind::Var,
            "let" => TokenKind::Let,
            "unsafe" => TokenKind::Unsafe,
            "asm" => TokenKind::Asm,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "trap" => TokenKind::Trap,
            "goto" => TokenKind::Goto,
            "module" => TokenKind::Module,
            "import" => TokenKind::Import,
            "struct" => TokenKind::Struct,
            "align" => TokenKind::Align,
            "cc" => TokenKind::Cc,
            "pure" => TokenKind::Pure,
            "noinline" => TokenKind::NoInline,
            "inline" => TokenKind::Inline,
            "cold" => TokenKind::Cold,
            "volatile" => TokenKind::Volatile,
            "default" => TokenKind::Default,
            "case" => TokenKind::Case,
            "global" => TokenKind::Global,
            "switch" => TokenKind::Switch,
            "null" => TokenKind::Null,
            _ => TokenKind::Ident(text.to_string()),
        };
        Ok(Token::new(kind, self.span_from(start_loc, start_byte), text.to_string()))
    }

    fn scan_number(&mut self, start: (Loc, usize)) -> Result<Token, LexError> {
        let start_byte = start.1;
        let start_loc = start.0;
        let mut hex = false;
        let mut bin = false;
        if self.bytes[self.pos] == b'0' && self.pos + 1 < self.bytes.len() {
            match self.bytes[self.pos + 1] {
                b'x' | b'X' => { hex = true; self.pos += 2; self.col += 2; }
                b'b' | b'B' => { bin = true; self.pos += 2; self.col += 2; }
                _ => {}
            }
        }
        let mut is_float = false;
        if !hex && !bin {
            while self.pos < self.bytes.len() {
                let b = self.bytes[self.pos];
                if b.is_ascii_digit() {
                    self.pos += 1;
                    self.col += 1;
                } else if b == b'.' && self.pos + 1 < self.bytes.len() && self.bytes[self.pos + 1].is_ascii_digit() {
                    is_float = true;
                    self.pos += 1;
                    self.col += 1;
                    while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
                        self.pos += 1;
                        self.col += 1;
                    }
                    break;
                } else {
                    break;
                }
            }
        } else {
            while self.pos < self.bytes.len() {
                let b = self.bytes[self.pos];
                let ok = if hex { b.is_ascii_hexdigit() } else { b == b'0' || b == b'1' };
                if ok {
                    self.pos += 1;
                    self.col += 1;
                } else {
                    break;
                }
            }
        }
        let text = &self.src[start_byte..self.pos];
        if is_float {
            let suffix = self.scan_float_suffix();
            Ok(Token::new(TokenKind::FloatLit(text.to_string(), suffix), self.span_from(start_loc, start_byte), text.to_string()))
        } else {
            let value = if hex { u64::from_str_radix(&text[2..], 16) } else if bin { u64::from_str_radix(&text[2..], 2) } else { text.parse::<u64>() }.map_err(|_| LexError::InvalidNumber(start_byte, text.to_string()))?;
            let mut suffix = IntSuffix::None;
            let rest = &self.src[self.pos..];
            for (s, suf) in [("i64", IntSuffix::I64), ("i32", IntSuffix::I32), ("i16", IntSuffix::I16), ("i8", IntSuffix::I8)] {
                if rest.starts_with(s) && (self.pos + s.len() >= self.bytes.len() || !self.is_id_cont(self.bytes[self.pos + s.len()])) {
                    suffix = suf;
                    self.pos += s.len();
                    self.col += s.len();
                    break;
                }
            }
            Ok(Token::new(TokenKind::IntLit(value, suffix), self.span_from(start_loc, start_byte), text.to_string()))
        }
    }

    fn scan_float_suffix(&mut self) -> FloatSuffix {
        let rest = &self.src[self.pos..];
        for (s, suf) in [("f64", FloatSuffix::F64), ("f32", FloatSuffix::F32)] {
            if rest.starts_with(s) && (self.pos + s.len() >= self.bytes.len() || !self.is_id_cont(self.bytes[self.pos + s.len()])) {
                self.pos += s.len();
                self.col += s.len();
                return suf;
            }
        }
        FloatSuffix::None
    }

    fn is_id_cont(&self, b: u8) -> bool {
        b.is_ascii_alphanumeric() || b == b'_'
    }

    fn scan_string(&mut self, start: (Loc, usize)) -> Result<Token, LexError> {
        let start_byte = start.1;
        let start_loc = start.0;
        // opening quote already at pos
        self.pos += 1; self.col += 1;
        let mut out = String::new();
        loop {
            if self.pos >= self.bytes.len() {
                return Err(LexError::UnterminatedString(start_byte));
            }
            let b = self.bytes[self.pos];
            if b == b'"' {
                self.pos += 1; self.col += 1;
                break;
            }
            if b == b'\\' {
                self.pos += 1; self.col += 1;
                if self.pos >= self.bytes.len() {
                    return Err(LexError::UnterminatedString(start_byte));
                }
                let eb = self.bytes[self.pos];
                let c = match eb {
                    b'n' => '\n',
                    b't' => '\t',
                    b'r' => '\r',
                    b'\\' => '\\',
                    b'"' => '"',
                    b'\'' => '\'',
                    b'0' => '\0',
                    _ => return Err(LexError::InvalidEscape(start_byte, eb as char)),
                };
                out.push(c);
                self.pos += 1; self.col += 1;
            } else {
                out.push(b as char);
                self.pos += 1; self.col += 1;
            }
        }
        Ok(Token::new(TokenKind::StringLit(out.clone()), self.span_from(start_loc, start_byte), out))
    }

    fn scan_char(&mut self, start: (Loc, usize)) -> Result<Token, LexError> {
        let start_byte = start.1;
        let start_loc = start.0;
        self.pos += 1; self.col += 1;
        if self.pos >= self.bytes.len() { return Err(LexError::UnterminatedString(start_byte)); }
        let b = self.bytes[self.pos];
        let c = if b == b'\\' {
            self.pos += 1; self.col += 1;
            if self.pos >= self.bytes.len() { return Err(LexError::UnterminatedString(start_byte)); }
            match self.bytes[self.pos] {
                b'n' => '\n', b't' => '\t', b'r' => '\r', b'\\' => '\\', b'\'' => '\'', b'0' => '\0',
                _ => return Err(LexError::InvalidEscape(start_byte, self.bytes[self.pos] as char)),
            }
        } else {
            b as char
        };
        self.pos += 1; self.col += 1;
        if self.pos >= self.bytes.len() || self.bytes[self.pos] != b'\'' {
            return Err(LexError::UnterminatedString(start_byte));
        }
        self.pos += 1; self.col += 1;
        Ok(Token::new(TokenKind::CharLit(c), self.span_from(start_loc, start_byte), c.to_string()))
    }

    fn scan_punct(&mut self, start: (Loc, usize)) -> Result<Option<Token>, LexError> {
        let start_byte = start.1;
        let start_loc = start.0;
        let b = self.bytes[self.pos];
        let two = self.bytes.get(self.pos + 1).copied();
        let kind = match b {
            b'(' => TokenKind::LParen,
            b')' => TokenKind::RParen,
            b'{' => TokenKind::LBrace,
            b'}' => TokenKind::RBrace,
            b'[' => TokenKind::LBracket,
            b']' => TokenKind::RBracket,
            b',' => TokenKind::Comma,
            b':' => TokenKind::Colon,
            b';' => TokenKind::Semicolon,
            b'!' => {
                if two == Some(b'=') { self.pos += 2; self.col += 2; TokenKind::Ne } else { return Err(LexError::InvalidChar(start_byte, '!')); }
            }
            b'=' => {
                if two == Some(b'=') { self.pos += 2; self.col += 2; TokenKind::Eq }
                else if two == Some(b'>') { self.pos += 2; self.col += 2; TokenKind::FatArrow }
                else { self.pos += 1; self.col += 1; TokenKind::Equals }
            }
            b'-' => {
                if two == Some(b'>') { self.pos += 2; self.col += 2; TokenKind::Arrow }
                else { self.pos += 1; self.col += 1; TokenKind::Minus }
            }
            b'.' => TokenKind::Dot,
            b'+' => TokenKind::Plus,
            b'*' => TokenKind::Star,
            b'/' => TokenKind::Slash,
            b'%' => TokenKind::Percent,
            b'<' => TokenKind::Lt,
            b'>' => TokenKind::Gt,
            _ => return Ok(None),
        };
        if b != b'=' && b != b'-' && b != b'!' { self.pos += 1; self.col += 1; }
        Ok(Some(Token::new(kind, self.span_from(start_loc, start_byte), b.to_string())))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexError {
    InvalidChar(usize, char),
    InvalidNumber(usize, String),
    UnterminatedString(usize),
    InvalidEscape(usize, char),
    UnterminatedBlockComment,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexError::InvalidChar(pos, c) => write!(f, "invalid character {:?} at byte {}", c, pos),
            LexError::InvalidNumber(pos, s) => write!(f, "invalid number {:?} at byte {}", s, pos),
            LexError::UnterminatedString(pos) => write!(f, "unterminated string at byte {}", pos),
            LexError::InvalidEscape(pos, c) => write!(f, "invalid escape {:?} at byte {}", c, pos),
            LexError::UnterminatedBlockComment => write!(f, "unterminated block comment"),
        }
    }
}

impl std::error::Error for LexError {}
