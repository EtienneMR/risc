use super::cursor::Cursor;
use super::error::TokenizationError;

pub fn read_number(cursor: &mut Cursor<'_>, start: usize) -> f64 {
    let mut found_dot = false;
    loop {
        match cursor.peek() {
            Some(b'0'..=b'9') => {
                cursor.bump();
            }
            Some(b'.') if !found_dot => {
                found_dot = true;
                cursor.bump();
            }
            _ => break,
        }
    }
    cursor
        .slice(start)
        .parse()
        .expect("slice should be a valid number")
}

pub fn read_string(cursor: &mut Cursor<'_>, start: usize) -> Result<String, TokenizationError> {
    let mut escaped = false;
    let mut bytes = Vec::new();

    loop {
        let byte = match cursor.bump() {
            Some(b) => b,
            None => {
                return Err(TokenizationError::new(
                    "unterminated string literal",
                    cursor.make_span(start),
                ))
            }
        };

        if escaped {
            bytes.push(match byte {
                b'n' => b'\n',
                b't' => b'\t',
                b'r' => b'\r',
                c => c,
            });
            escaped = false;
        } else {
            match byte {
                b'"' => break,
                b'\\' => escaped = true,
                c => bytes.push(c),
            }
        }
    }

    String::from_utf8(bytes).map_err(|_| {
        TokenizationError::new(
            "string literal contains invalid UTF-8",
            cursor.make_span(start),
        )
    })
}

pub fn read_identifier(cursor: &mut Cursor<'_>, start: usize) -> String {
    while matches!(
        cursor.peek(),
        Some(b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_')
    ) {
        cursor.bump();
    }
    cursor.slice(start).to_string()
}
