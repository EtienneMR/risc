use crate::{
    parser::{
        ast::{Expr, ExprKind},
        cursor::Cursor,
        error::ParseError,
        expr::{infix::parse_infix, prefix::parse_prefix},
    },
    tokenizer::{Symbol, Token, TokenKind},
};

pub fn parse_postfix(cursor: &mut Cursor) -> Result<Expr, ParseError> {
    let mut expr = parse_prefix(cursor)?;

    loop {
        if let Token {
            kind: TokenKind::Symbol(sym),
            span: _,
        } = cursor.peek()?
        {
            match sym {
                Symbol::LParen => {
                    expr = parse_call(cursor, expr)?;
                    continue;
                }
                Symbol::Dot => {
                    expr = parse_property(cursor, expr)?;
                    continue;
                }
                Symbol::LBracket => {
                    expr = parse_index(cursor, expr)?;
                    continue;
                }
                _ => {}
            }
        }
        return Ok(expr);
    }
}

fn parse_call(cursor: &mut Cursor, callee: Expr) -> Result<Expr, ParseError> {
    cursor.bump()?;

    let mut args = Vec::new();
    if !cursor.peek_kind(Symbol::RParen)? {
        loop {
            args.push(parse_infix(cursor)?);
            if cursor.peek_kind(Symbol::Comma)? {
                cursor.bump()?;
            } else {
                break;
            }
        }
    }

    let close = cursor.expect_kind(Symbol::RParen)?;
    let span = callee.span.merge(close);

    Ok(Expr::new(
        ExprKind::Call {
            callee: Box::new(callee),
            args,
        },
        span,
    ))
}

fn parse_property(cursor: &mut Cursor, object: Expr) -> Result<Expr, ParseError> {
    cursor.bump()?;
    let name_tok = cursor.bump()?;
    let property = match name_tok.kind {
        TokenKind::Identifier(n) => n,
        _ => return Err(ParseError::expected("identifier", &name_tok)),
    };
    let span = object.span.merge(name_tok.span);
    Ok(Expr::new(
        ExprKind::Index {
            object: Box::new(object),
            key: Box::new(Expr::new(ExprKind::String(property), name_tok.span)),
        },
        span,
    ))
}

fn parse_index(cursor: &mut Cursor, object: Expr) -> Result<Expr, ParseError> {
    cursor.bump()?;
    let key = parse_infix(cursor)?;
    let close = cursor.expect_kind(Symbol::RBracket)?;
    let span = object.span.merge(close);
    Ok(Expr::new(
        ExprKind::Index {
            object: Box::new(object),
            key: Box::new(key),
        },
        span,
    ))
}
