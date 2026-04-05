use crate::{
    parser::{
        ast::{BinaryOp, Expr, ExprKind},
        cursor::Cursor,
        error::ParseError,
        expr::postfix::parse_postfix,
    },
    tokenizer::{Symbol, TokenKind},
};

pub fn parse_infix(cursor: &mut Cursor) -> Result<Expr, ParseError> {
    parse_bp(cursor, 0)
}

fn parse_bp(cursor: &mut Cursor, min_bp: u8) -> Result<Expr, ParseError> {
    let mut lhs = parse_postfix(cursor)?;

    loop {
        let Some((l_bp, r_bp, op)) = infix_binding_power(cursor)? else {
            break;
        };
        if l_bp < min_bp {
            break;
        }

        cursor.bump()?;
        let rhs = parse_bp(cursor, r_bp)?;
        let span = lhs.span.merge(rhs.span);
        lhs = Expr::new(
            ExprKind::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            span,
        );
    }

    Ok(lhs)
}

fn infix_binding_power(cursor: &mut Cursor) -> Result<Option<(u8, u8, BinaryOp)>, ParseError> {
    Ok(match cursor.peek()?.kind {
        TokenKind::Symbol(Symbol::Eq) => Some((1, 1, BinaryOp::Assign)),
        TokenKind::Symbol(Symbol::PipeGt) => Some((2, 3, BinaryOp::Pipe)),
        TokenKind::Symbol(Symbol::OrOr) => Some((3, 4, BinaryOp::Or)),
        TokenKind::Symbol(Symbol::AndAnd) => Some((4, 5, BinaryOp::And)),
        TokenKind::Symbol(Symbol::EqEq) => Some((5, 6, BinaryOp::Eq)),
        TokenKind::Symbol(Symbol::NotEq) => Some((5, 6, BinaryOp::NotEq)),
        TokenKind::Symbol(Symbol::Lt) => Some((6, 7, BinaryOp::Lt)),
        TokenKind::Symbol(Symbol::Lte) => Some((6, 7, BinaryOp::Lte)),
        TokenKind::Symbol(Symbol::Gt) => Some((6, 7, BinaryOp::Gt)),
        TokenKind::Symbol(Symbol::Gte) => Some((6, 7, BinaryOp::Gte)),
        TokenKind::Symbol(Symbol::Plus) => Some((7, 8, BinaryOp::Add)),
        TokenKind::Symbol(Symbol::Minus) => Some((7, 8, BinaryOp::Sub)),
        TokenKind::Symbol(Symbol::Star) => Some((8, 9, BinaryOp::Mul)),
        TokenKind::Symbol(Symbol::Slash) => Some((8, 9, BinaryOp::Div)),
        TokenKind::Symbol(Symbol::Percent) => Some((8, 9, BinaryOp::Rem)),
        _ => None,
    })
}
