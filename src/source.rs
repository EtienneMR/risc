#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SourceId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub source: SourceId,
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(source: SourceId, start: usize, end: usize) -> Self {
        Self { source, start, end }
    }

    pub fn merge(self, other: Self) -> Self {
        debug_assert_eq!(self.source, other.source, "cannot merge to different files");
        Self {
            source: self.source,
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

pub struct Source {
    pub id: SourceId,
    pub file: String,
    pub content: String,
}

impl Source {
    pub fn new(id: SourceId, file: String, content: String) -> Self {
        Self { id, file, content }
    }

    fn byte_to_line_col(&self, byte: usize) -> (usize, usize) {
        let prefix = &self.content[..byte];
        let line = prefix.bytes().filter(|&b| b == b'\n').count();
        let col = prefix.rfind('\n').map(|nl| byte - nl - 1).unwrap_or(byte);
        (line, col)
    }

    pub fn with_context(&self, span: Span, message: &str) -> String {
        let (line, col) = self.byte_to_line_col(span.start);

        let mut out = format!("{}:{}:{}: {}\n", self.file, line + 1, col + 1, message,);

        let context_radius: usize = 2;
        let first_shown = line.saturating_sub(context_radius);
        let last_shown = line + context_radius;

        for (abs_line, source_line) in self
            .content
            .lines()
            .enumerate()
            .skip(first_shown)
            .take_while(|(n, _)| *n <= last_shown)
        {
            let line_number = abs_line + 1;
            out.push_str(&format!("{line_number:>4} | {source_line}\n"));

            if abs_line == line {
                let padding = " ".repeat(col);
                let underline = "^".repeat((span.end - span.start).max(1));
                out.push_str(&format!("     | {padding}{underline}\n"));
            }
        }

        out
    }
}
