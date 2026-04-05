use crate::source::{Source, Span};
pub struct Cursor<'a> {
    source: &'a Source,
    bytes: &'a [u8],
    index: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(source: &'a Source) -> Self {
        Self {
            source,
            bytes: source.content.as_bytes(),
            index: 0,
        }
    }

    #[inline]
    pub fn peek(&self) -> Option<u8> {
        self.bytes.get(self.index).copied()
    }

    #[inline]
    pub fn peek_next(&self) -> Option<u8> {
        self.bytes.get(self.index + 1).copied()
    }

    #[inline]
    pub fn bump(&mut self) -> Option<u8> {
        let b = self.peek()?;
        self.index += 1;
        Some(b)
    }

    #[inline]
    pub fn pos(&self) -> usize {
        self.index
    }

    #[inline]
    pub fn slice(&self, start: usize) -> &'a str {
        &self.source.content[start..self.index]
    }

    #[inline]
    pub fn make_span(&self, start: usize) -> Span {
        Span::new(self.source.id, start, self.index)
    }

    pub fn skip_whitespace_and_comments(&mut self) {
        loop {
            while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
                self.index += 1;
            }
            if self.peek() == Some(b'-') && self.peek_next() == Some(b'-') {
                while !matches!(self.bump(), None | Some(b'\n')) {}
            } else {
                break;
            }
        }
    }
}
