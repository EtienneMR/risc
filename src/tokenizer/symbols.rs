use super::cursor::Cursor;
use super::token::Symbol;

pub fn read_symbol(cursor: &mut Cursor<'_>, byte: u8) -> Option<Symbol> {
    Some(match byte {
        b'(' => Symbol::LParen,
        b')' => Symbol::RParen,
        b'{' => Symbol::LBrace,
        b'}' => Symbol::RBrace,
        b'[' => Symbol::LBracket,
        b']' => Symbol::RBracket,

        b',' => Symbol::Comma,
        b'.' => Symbol::Dot,
        b':' => Symbol::Colon,
        b';' => Symbol::Semicolon,

        b'+' => Symbol::Plus,
        b'*' => Symbol::Star,
        b'/' => Symbol::Slash,
        b'%' => Symbol::Percent,
        b'?' => Symbol::Question,

        b'-' => {
            if cursor.peek() == Some(b'>') {
                cursor.bump();
                Symbol::Arrow
            } else {
                Symbol::Minus
            }
        }
        b'=' => {
            if cursor.peek() == Some(b'=') {
                cursor.bump();
                Symbol::EqEq
            } else {
                Symbol::Eq
            }
        }
        b'!' => {
            if cursor.peek() == Some(b'=') {
                cursor.bump();
                Symbol::NotEq
            } else {
                Symbol::Not
            }
        }
        b'<' => {
            if cursor.peek() == Some(b'=') {
                cursor.bump();
                Symbol::Lte
            } else {
                Symbol::Lt
            }
        }
        b'>' => {
            if cursor.peek() == Some(b'=') {
                cursor.bump();
                Symbol::Gte
            } else {
                Symbol::Gt
            }
        }
        b'&' => {
            if cursor.peek() == Some(b'&') {
                cursor.bump();
                Symbol::AndAnd
            } else {
                return None;
            }
        }
        b'|' => match cursor.peek() {
            Some(b'|') => {
                cursor.bump();
                Symbol::OrOr
            }
            Some(b'>') => {
                cursor.bump();
                Symbol::PipeGt
            }
            _ => Symbol::Pipe,
        },

        _ => return None,
    })
}
