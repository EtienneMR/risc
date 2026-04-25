//! SourceMap accumulates named source strings and assigns each a stable SourceId.
//! Span records a half-open byte range [start, end) within a specific SourceId.
//! render_span_context() formats an error caret with surrounding numbered source lines.
//! SourceId::INTERNAL is the synthetic "<internal>" source used for built-in call spans.
//! Sources are append-only; SourceIds remain valid for the lifetime of the SourceMap.

pub struct SourceMap {
    sources: Vec<Source>,
}

#[derive(Debug, Clone, Copy)]
pub struct Location {
    pub line: usize,
    pub col: usize,
}

impl SourceMap {
    pub fn new() -> Self {
        Self {
            sources: vec![Source {
                id: SourceId::INTERNAL,
                name: "<internal>".into(),
                content: String::new(),
            }],
        }
    }

    pub fn add(&mut self, name: String, content: String) -> &Source {
        self.sources.push(Source {
            id: SourceId(self.sources.len()),
            name,
            content,
        });
        self.sources.last().expect("source should have been pushed")
    }

    pub fn get(&self, id: SourceId) -> &Source {
        self.sources
            .get(id.0)
            .expect("source id should remain valid")
    }

    pub fn get_location(&self, span: Span) -> Location {
        let source = self.get(span.source);
        let prefix = &source.content[..span.start];
        let line = prefix.bytes().filter(|&b| b == b'\n').count();
        let col = prefix
            .rfind('\n')
            .map(|pos| span.start - pos - 1)
            .unwrap_or(span.start);
        Location { line, col }
    }

    pub fn format_location(&self, span: Span) -> String {
        let source = self.get(span.source);
        let loc = self.get_location(span);
        format!("{}:{}:{}", source.name, loc.line + 1, loc.col + 1)
    }

    pub fn extract_line(&self, span: Span) -> Option<&str> {
        let source = self.get(span.source);
        let loc = self.get_location(span);

        source
            .content
            .lines()
            .skip(loc.line)
            .next()
            .map(|s| s.trim())
    }

    pub fn render_span_context(
        &self,
        span: Span,
        pre_content: usize,
        post_content: usize,
    ) -> String {
        let source = self.get(span.source);
        let loc = self.get_location(span);
        let mut out = String::new();

        let first_shown = loc.line.saturating_sub(pre_content);
        let last_shown = loc.line + post_content;

        let padding_width = (last_shown.checked_ilog10().unwrap_or(0) + 1) as usize;

        for (abs_line, source_line) in source
            .content
            .lines()
            .enumerate()
            .skip(first_shown)
            .take_while(|(n, _)| *n <= last_shown)
        {
            let line_number = abs_line + 1;
            out.push_str(&format!(
                "{line_number:>width$} | {source_line}\n",
                width = padding_width
            ));

            if abs_line == loc.line {
                let padding = " ".repeat(loc.col);
                let underline = "^".repeat((span.end - span.start).max(1));
                out.push_str(&format!(
                    "{} | {padding}{underline}\n",
                    " ".repeat(padding_width)
                ));
            }
        }

        out.pop(); // final \n
        out
    }
}

pub struct Source {
    pub id: SourceId,
    pub name: String,
    pub content: String,
}

impl Source {
    pub fn create_span(&self, start: usize, end: usize) -> Span {
        assert!(start <= end);
        assert!(end <= self.content.len());
        Span {
            source: self.id,
            start,
            end,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceId(usize);

impl SourceId {
    pub const INTERNAL: Self = Self(0);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub source: SourceId,
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn merge(self, other: Self) -> Self {
        assert_eq!(self.source, other.source);
        Self {
            source: self.source,
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}
