use std::fmt::Display;

pub struct SourceMap {
    sources: Vec<Source>,
}

impl SourceMap {
    const CONTEXT_RADIUS: usize = 3;

    pub fn new() -> Self {
        Self {
            sources: vec![Source {
                id: SourceId(0),
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

    pub fn with_context(&self, span: Span, message: impl Display) -> String {
        let source = self.get(span.source);

        let prefix = &source.content[..span.start];
        let line = prefix.bytes().filter(|&b| b == b'\n').count();
        let col = prefix
            .rfind('\n')
            .map(|pos| span.start - pos - 1)
            .unwrap_or(span.start);

        let mut out = format!("{}:{}:{}: {}\n", source.name, line + 1, col + 1, message);

        let first_shown = line.saturating_sub(Self::CONTEXT_RADIUS);
        let last_shown = line + Self::CONTEXT_RADIUS;

        for (abs_line, source_line) in source
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
    pub const INTERNAL: Self = Self {
        source: SourceId::INTERNAL,
        start: 0,
        end: 0,
    };

    pub fn merge(self, other: Self) -> Self {
        assert_eq!(self.source, other.source);
        Self {
            source: self.source,
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}
