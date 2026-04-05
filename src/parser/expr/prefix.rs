use crate::{
    parser::{
        ast::{Expr, ExprKind, TableRow, UnaryOp},
        cursor::Cursor,
        error::ParseError,
        expr::{block::parse_block, infix::parse_infix, postfix::parse_postfix},
    },
    source::Span,
    tokenizer::{Keyword, Symbol, Token, TokenKind},
};

fn can_begin_expr(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Number(_)
            | TokenKind::String(_)
            | TokenKind::Identifier(_)
            | TokenKind::Keyword(
                Keyword::True
                    | Keyword::False
                    | Keyword::Nil
                    | Keyword::If
                    | Keyword::For
                    | Keyword::While
                    | Keyword::Let
                    | Keyword::Fn
            )
            | TokenKind::Symbol(Symbol::LParen | Symbol::LBrace | Symbol::Minus | Symbol::Not)
    )
}

pub fn parse_prefix(cursor: &mut Cursor) -> Result<Expr, ParseError> {
    let tok = cursor.bump()?;
    let span = tok.span;

    match tok.kind {
        TokenKind::Number(n) => Ok(Expr::new(ExprKind::Number(n), span)),
        TokenKind::String(s) => Ok(Expr::new(ExprKind::String(s), span)),
        TokenKind::Identifier(n) => Ok(Expr::new(ExprKind::Identifier(n), span)),

        TokenKind::Keyword(Keyword::True) => Ok(Expr::new(ExprKind::Bool(true), span)),
        TokenKind::Keyword(Keyword::False) => Ok(Expr::new(ExprKind::Bool(false), span)),
        TokenKind::Keyword(Keyword::Nil) => Ok(Expr::new(ExprKind::Nil, span)),

        TokenKind::Symbol(Symbol::LParen) => {
            let inner = parse_infix(cursor)?;
            let close = cursor.expect_kind(Symbol::RParen)?;
            Ok(Expr::new(inner.kind, span.merge(close)))
        }

        TokenKind::Symbol(Symbol::LBrace) => parse_table(cursor, span),

        TokenKind::Keyword(Keyword::If) => parse_if(cursor, span),
        TokenKind::Keyword(Keyword::For) => parse_for(cursor, span),
        TokenKind::Keyword(Keyword::While) => parse_while(cursor, span),
        TokenKind::Keyword(Keyword::Let) => parse_bind(cursor, span),
        TokenKind::Keyword(Keyword::Fn) => parse_function(cursor, span),

        TokenKind::Keyword(Keyword::Break) => Ok(Expr::new(ExprKind::Break, span)),
        TokenKind::Keyword(Keyword::Continue) => Ok(Expr::new(ExprKind::Continue, span)),
        TokenKind::Keyword(Keyword::Return) => {
            let value = if can_begin_expr(&cursor.peek()?.kind) {
                parse_infix(cursor)?
            } else {
                Expr::new(ExprKind::Nil, span)
            };
            let end = span.merge(value.span);
            Ok(Expr::new(ExprKind::Return(Box::new(value)), end))
        }

        TokenKind::Symbol(Symbol::Minus) => {
            let rhs = parse_postfix(cursor)?;
            let full = span.merge(rhs.span);
            Ok(Expr::new(
                ExprKind::Unary {
                    op: UnaryOp::Neg,
                    rhs: Box::new(rhs),
                },
                full,
            ))
        }

        TokenKind::Symbol(Symbol::Not) => {
            let rhs = parse_postfix(cursor)?;
            let full = span.merge(rhs.span);
            Ok(Expr::new(
                ExprKind::Unary {
                    op: UnaryOp::Not,
                    rhs: Box::new(rhs),
                },
                full,
            ))
        }

        other => Err(ParseError::new(
            format!("unexpected token: {other:?}"),
            span,
        )),
    }
}

fn parse_if(cursor: &mut Cursor, open_span: Span) -> Result<Expr, ParseError> {
    let condition = parse_infix(cursor)?;
    cursor.expect_kind(Keyword::Do)?;

    let then_branch = parse_block(
        cursor,
        &[
            TokenKind::Keyword(Keyword::Else),
            TokenKind::Keyword(Keyword::End),
        ],
    )?;

    let else_branch = if cursor.peek()?.kind == TokenKind::Keyword(Keyword::Else) {
        cursor.expect_kind(Keyword::Else)?;
        Some(parse_block(cursor, &[TokenKind::Keyword(Keyword::End)])?)
    } else {
        None
    };

    let end = cursor.expect_kind(Keyword::End)?;

    Ok(Expr::new(
        ExprKind::If {
            condition: Box::new(condition),
            then_branch,
            else_branch,
        },
        open_span.merge(end),
    ))
}

fn parse_for(cursor: &mut Cursor, open_span: Span) -> Result<Expr, ParseError> {
    let name_tok = cursor.bump()?;
    let Token {
        kind: TokenKind::Identifier(identifier),
        ..
    } = name_tok
    else {
        return Err(ParseError::expected("identifier", &name_tok));
    };

    cursor.expect_kind(Keyword::In)?;

    let iterator = parse_infix(cursor)?;
    cursor.expect_kind(Keyword::Do)?;
    let body = parse_block(cursor, &[TokenKind::Keyword(Keyword::End)])?;
    let end = cursor.expect_kind(Keyword::End)?;

    Ok(Expr::new(
        ExprKind::For {
            identifier,
            iterator: Box::new(iterator),
            body,
        },
        open_span.merge(end),
    ))
}

fn parse_while(cursor: &mut Cursor, open_span: Span) -> Result<Expr, ParseError> {
    let condition = parse_infix(cursor)?;
    cursor.expect_kind(Keyword::Do)?;
    let body = parse_block(cursor, &[TokenKind::Keyword(Keyword::End)])?;
    let end = cursor.expect_kind(Keyword::End)?;

    Ok(Expr::new(
        ExprKind::While {
            condition: Box::new(condition),
            body,
        },
        open_span.merge(end),
    ))
}

fn parse_table(cursor: &mut Cursor, open_span: Span) -> Result<Expr, ParseError> {
    let mut rows = Vec::new();

    while !cursor.peek_kind(Symbol::RBrace)? {
        let key = parse_table_key(cursor, open_span, rows.len())?;
        let value = parse_infix(cursor)?;
        rows.push(TableRow {
            key: Box::new(key),
            value: Box::new(value),
        });

        if cursor.peek_kind(Symbol::Comma)? {
            cursor.bump()?;
        } else {
            break;
        }
    }

    let close = cursor.expect_kind(Symbol::RBrace)?;
    Ok(Expr::new(ExprKind::Table(rows), open_span.merge(close)))
}

fn parse_table_key(
    cursor: &mut Cursor,
    open_span: Span,
    row_index: usize,
) -> Result<Expr, ParseError> {
    if cursor.peek_kind(Symbol::LBracket)? {
        cursor.bump()?;
        let key = parse_infix(cursor)?;
        cursor.expect_kind(Symbol::RBracket)?;
        cursor.expect_kind(Symbol::Eq)?;
        return Ok(key);
    }

    if cursor.peek_kind(Symbol::Dot)? {
        cursor.bump()?;
        let name_tok = cursor.bump()?;
        let Token {
            kind: TokenKind::Identifier(name),
            span: name_span,
        } = name_tok
        else {
            return Err(ParseError::expected("identifier", &name_tok));
        };
        cursor.expect_kind(Symbol::Eq)?;
        return Ok(Expr::new(ExprKind::String(name), name_span));
    }

    let positional_span = Span::new(open_span.source, open_span.start, open_span.start);
    Ok(Expr::new(
        ExprKind::Number(row_index as f64),
        positional_span,
    ))
}

fn parse_bind(cursor: &mut Cursor, open_span: Span) -> Result<Expr, ParseError> {
    let is_function = if cursor.peek_kind(Keyword::Fn)? {
        cursor.bump()?;
        true
    } else {
        false
    };

    let identifier_token = cursor.bump()?;
    let Token {
        kind: TokenKind::Identifier(identifier),
        ..
    } = identifier_token
    else {
        return Err(ParseError::expected("identifier", &identifier_token));
    };

    let value = if is_function {
        parse_function(cursor, open_span)?
    } else {
        cursor.expect_kind(Symbol::Eq)?;
        parse_infix(cursor)?
    };

    let span = open_span.merge(value.span);

    Ok(Expr::new(
        ExprKind::Bind { identifier, value: Box::new(value) },
        span,
    ))
}

fn parse_function(cursor: &mut Cursor, open_span: Span) -> Result<Expr, ParseError> {
    cursor.expect_kind(Symbol::LParen)?;

    let mut params = Vec::new();
    if !cursor.peek_kind(Symbol::RParen)? {
        loop {
            let param_token = cursor.bump()?;
            let Token {
                kind: TokenKind::Identifier(param_name),
                ..
            } = param_token
            else {
                return Err(ParseError::expected("identifier", &param_token));
            };

            params.push(param_name);
            if cursor.peek_kind(Symbol::Comma)? {
                cursor.bump()?;
            } else {
                break;
            }
        }
    }
    cursor.expect_kind(Symbol::RParen)?;

    let body = parse_block(cursor, &[TokenKind::Keyword(Keyword::End)])?;
    let end = cursor.expect_kind(Keyword::End)?;

    Ok(Expr {
        kind: ExprKind::Function { params, body },
        span: open_span.merge(end),
    })
}
