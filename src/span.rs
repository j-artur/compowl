#[derive(Debug, Clone)]
pub struct Source {
    pub filename: String,
    pub content: String,
}

#[derive(Clone, Copy)]
pub struct Span<'s> {
    start: usize,
    end: usize,
    src: &'s Source,
}

impl<'s> Span<'s> {
    pub fn fragment(&self) -> &'s str {
        &self.src.content[self.start..self.end]
    }

    pub fn line(&self) -> usize {
        // line numbers are 1-indexed
        // get the number of newlines before the start of the span

        self.src.content[..self.start].matches('\n').count() + 1
    }

    pub fn column(&self) -> usize {
        // column numbers are 1-indexed
        // get the number of characters between the last newline and the start of the span

        let last_newline = self.src.content[..self.start]
            .rfind('\n')
            .map(|i| i + 1)
            .unwrap_or(0);

        self.start - last_newline + 1
    }

    pub fn location(&self) -> String {
        format!("{}:{}:{}", self.src.filename, self.line(), self.column())
    }
}

impl Span<'_> {
    pub fn take(&self, n: usize) -> Self {
        Self {
            start: self.start,
            end: self.start + n,
            src: self.src,
        }
    }

    pub fn shift(&self, n: usize) -> Self {
        Self {
            start: self.start + n,
            end: self.end,
            src: self.src,
        }
    }

    pub fn split(&self, n: usize) -> (Self, Self) {
        let taken = self.take(n);
        let remaining = self.shift(n);
        (remaining, taken)
    }

    pub fn merge(&self, other: &Self) -> Self {
        Self {
            start: self.start,
            end: other.end,
            src: self.src,
        }
    }
}

impl<'s> From<&'s Source> for Span<'s> {
    fn from(src: &'s Source) -> Self {
        Self {
            start: 0,
            end: src.content.len(),
            src,
        }
    }
}

pub struct Located<'s, T> {
    pub value: T,
    pub span: Span<'s>,
}

impl<'s, T> Located<'s, T> {
    pub fn new(value: T, span: Span<'s>) -> Self {
        Self { value, span }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Located<'s, U> {
        Located::new(f(self.value), self.span)
    }
}

impl<'s, T: std::fmt::Debug> std::fmt::Debug for Located<'s, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.span.location(), self.value)
    }
}
