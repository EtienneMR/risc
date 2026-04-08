use super::token::{Keyword, TokenKind};

pub fn keyword(ident: &str) -> Option<TokenKind> {
    Some(TokenKind::Keyword(match ident {
        "do" => Keyword::Do,
        "end" => Keyword::End,
        "if" => Keyword::If,
        "else" => Keyword::Else,
        "for" => Keyword::For,
        "in" => Keyword::In,
        "while" => Keyword::While,
        "return" => Keyword::Return,
        "break" => Keyword::Break,
        "continue" => Keyword::Continue,
        "try" => Keyword::Try,
        "catch" => Keyword::Catch,
        "as" => Keyword::As,
        "let" => Keyword::Let,
        "fn" => Keyword::Fn,
        "nil" => Keyword::Nil,
        "true" => Keyword::True,
        "false" => Keyword::False,
        _ => return None,
    }))
}
