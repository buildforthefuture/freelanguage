use std::fmt;

/// A byte offset into a source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Offset(pub usize);

/// A source location (line, column) 1-indexed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Loc {
    pub line: usize,
    pub col: usize,
}

/// A span over source text, half-open [start, end).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Span {
    pub start: Loc,
    pub end: Loc,
    pub byte_start: usize,
    pub byte_end: usize,
}

impl Span {
    pub fn none() -> Self {
        Span {
            start: Loc { line: 0, col: 0 },
            end: Loc { line: 0, col: 0 },
            byte_start: 0,
            byte_end: 0,
        }
    }

    pub fn merge(a: Span, b: Span) -> Self {
        let byte_start = a.byte_start.min(b.byte_start);
        let byte_end = a.byte_end.max(b.byte_end);
        Span {
            start: if a.byte_start <= b.byte_start { a.start } else { b.start },
            end: if a.byte_end >= b.byte_end { a.end } else { b.end },
            byte_start,
            byte_end,
        }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}-{}:{}", self.start.line, self.start.col, self.end.line, self.end.col)
    }
}

/// A diagnostic produced by the lexer, parser, or verifier.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub code: String,
    pub span: Span,
    pub message: String,
    pub hint: Option<String>,
}

impl Diagnostic {
    pub fn new(code: impl Into<String>, span: Span, message: impl Into<String>) -> Self {
        Diagnostic {
            code: code.into(),
            span,
            message: message.into(),
            hint: None,
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}: {}: {}", self.code, self.span.start.line, self.span.start.col, self.span, self.message)?;
        if let Some(h) = &self.hint {
            write!(f, " (hint: {})", h)?;
        }
        Ok(())
    }
}

/// A collectable list of diagnostics.
#[derive(Debug, Default, Clone)]
pub struct Diagnostics {
    pub errors: Vec<Diagnostic>,
}

impl Diagnostics {
    pub fn add(&mut self, d: Diagnostic) {
        self.errors.push(d);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

/// Result type used during parsing: either an OK value or a set of diagnostics.
#[derive(Debug)]
pub struct ParseResult<T> {
    pub value: Option<T>,
    pub diags: Diagnostics,
}

impl<T> ParseResult<T> {
    pub fn ok(value: T) -> Self {
        ParseResult { value: Some(value), diags: Diagnostics::default() }
    }

    pub fn err(diag: Diagnostic) -> Self {
        ParseResult { value: None, diags: Diagnostics { errors: vec![diag] } }
    }

    pub fn merge(mut self, other: ParseResult<T>) -> Self {
        self.diags.errors.extend(other.diags.errors);
        if other.value.is_some() {
            self.value = other.value;
        }
        self
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> ParseResult<U> {
        ParseResult {
            value: self.value.map(f),
            diags: self.diags,
        }
    }
}
