//! Pratt (top-down operator precedence) expression parser.
//! parse_precedence(min_bp) drives the loop; binding powers in BP_* constants control associativity.
//! Postfix forms (call, dot-access, bracket-index) are handled inline in parse_precedence.
//! Prefix forms: unary minus and "not". Primary: literals, identifiers, grouped, and table literals.
//! Keyword expressions (if, for, while, fn, let, try) are dispatched from parse_primary.

use crate::{
    ast::{BinaryOp, NodeId, NodeKind, UnaryOp},
    error::LangError,
    lexer::{Keyword, Symbol, TokenKind},
};

const BP_LOWEST: u8 = 0;
const BP_ASSIGN: u8 = 10;
const BP_PIPE: u8 = 20;
const BP_OR: u8 = 30;
const BP_AND: u8 = 40;
const BP_EQUALITY: u8 = 50;
const BP_COMPARISON: u8 = 60;
const BP_SUM: u8 = 70;
const BP_PRODUCT: u8 = 80;
const BP_PREFIX: u8 = 90;
const BP_POSTFIX: u8 = 100;

const fn left_assoc(bp: u8) -> (u8, u8) {
    (bp, bp + 1)
}

const fn right_assoc(bp: u8) -> (u8, u8) {
    (bp, bp - 1)
}

impl<'a> super::Parser<'a> {
    pub(super) fn parse_expression(&mut self) -> Result<NodeId, LangError> {
        self.parse_precedence(BP_LOWEST)
    }

    fn parse_precedence(&mut self, min_bp: u8) -> Result<NodeId, LangError> {
        let mut left = self.parse_prefix()?;

        loop {
            if self.peek_kind(Symbol::LParen)? {
                if BP_POSTFIX < min_bp {
                    break;
                }
                self.take()?;
                left = self.parse_call(left)?;
                continue;
            }

            if self.peek_kind(Symbol::Dot)? {
                if BP_POSTFIX < min_bp {
                    break;
                }
                self.take()?;
                left = self.parse_property(left)?;
                continue;
            }

            if self.peek_kind(Symbol::LBracket)? {
                if BP_POSTFIX < min_bp {
                    break;
                }
                self.take()?;
                left = self.parse_index(left)?;
                continue;
            }

            let Some(op) = self.peek_binary_op()? else {
                break;
            };

            let (l_bp, r_bp) = Self::binary_binding_power(op);
            if l_bp < min_bp {
                break;
            }

            self.take()?;
            let right = self.parse_precedence(r_bp)?;
            let span = self.ast.get(left).span.merge(self.ast.get(right).span);

            left = self.ast.add(NodeKind::Binary { op, left, right }, span);
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<NodeId, LangError> {
        if self.peek_kind(Symbol::Minus)? {
            let op = self.take()?;
            let right = self.parse_precedence(BP_PREFIX)?;
            let full = op.span.merge(self.ast.get(right).span);
            return Ok(self.ast.add(
                NodeKind::Unary {
                    op: UnaryOp::Neg,
                    right,
                },
                full,
            ));
        }

        if self.peek_kind(Symbol::Not)? {
            let op = self.take()?;
            let right = self.parse_precedence(BP_PREFIX)?;
            let full = op.span.merge(self.ast.get(right).span);
            return Ok(self.ast.add(
                NodeKind::Unary {
                    op: UnaryOp::Not,
                    right,
                },
                full,
            ));
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<NodeId, LangError> {
        let tok = self.take()?;
        let span = tok.span;

        match tok.kind {
            TokenKind::Number(n) => Ok(self.ast.add(NodeKind::Number(n), span)),
            TokenKind::String(s) => Ok(self.ast.add(NodeKind::String(s), span)),
            TokenKind::Identifier(n) => Ok(self.ast.add(NodeKind::Identifier(n), span)),

            TokenKind::Keyword(Keyword::True) => Ok(self.ast.add(NodeKind::Boolean(true), span)),
            TokenKind::Keyword(Keyword::False) => Ok(self.ast.add(NodeKind::Boolean(false), span)),
            TokenKind::Keyword(Keyword::Nil) => Ok(self.ast.add(NodeKind::Nil, span)),

            TokenKind::Symbol(Symbol::LParen) => {
                let inner = self.parse_block(span, &[TokenKind::Symbol(Symbol::RParen)])?;
                self.expect_kind(Symbol::RParen)?;
                Ok(inner)
            }

            TokenKind::Symbol(Symbol::LBrace) => self.parse_table(span),

            TokenKind::Keyword(Keyword::If) => self.parse_if(span),
            TokenKind::Keyword(Keyword::For) => self.parse_for(span),
            TokenKind::Keyword(Keyword::While) => self.parse_while(span),
            TokenKind::Keyword(Keyword::Let) => self.parse_declaration(span),
            TokenKind::Keyword(Keyword::Fn) => self.parse_function(span),

            TokenKind::Keyword(Keyword::Break) => {
                let value = self.parse_expression()?;
                let end = self.ast.get(value).span.merge(span);
                Ok(self.ast.add(NodeKind::Break(value), end))
            }
            TokenKind::Keyword(Keyword::Continue) => Ok(self.ast.add(NodeKind::Continue, span)),
            TokenKind::Keyword(Keyword::Return) => {
                let value = self.parse_expression()?;
                let end = self.ast.get(value).span.merge(span);
                Ok(self.ast.add(NodeKind::Return(value), end))
            }
            TokenKind::Keyword(Keyword::Try) => self.parse_try_catch(span),

            other => Err(LangError::expected_token("expression", &other, span)),
        }
        .map_err(|e| e.add_context(span))
    }

    fn peek_binary_op(&mut self) -> Result<Option<BinaryOp>, LangError> {
        Ok(Self::token_to_binary_op(&self.peek()?.kind))
    }

    fn token_to_binary_op(kind: &TokenKind) -> Option<BinaryOp> {
        match kind {
            TokenKind::Symbol(Symbol::Eq) => Some(BinaryOp::Assign),
            TokenKind::Symbol(Symbol::PipeGt) => Some(BinaryOp::Pipe),

            TokenKind::Symbol(Symbol::OrOr) => Some(BinaryOp::Or),
            TokenKind::Symbol(Symbol::AndAnd) => Some(BinaryOp::And),

            TokenKind::Symbol(Symbol::EqEq) => Some(BinaryOp::Eq),
            TokenKind::Symbol(Symbol::NotEq) => Some(BinaryOp::NotEq),

            TokenKind::Symbol(Symbol::Lt) => Some(BinaryOp::Lt),
            TokenKind::Symbol(Symbol::Lte) => Some(BinaryOp::Lte),
            TokenKind::Symbol(Symbol::Gt) => Some(BinaryOp::Gt),
            TokenKind::Symbol(Symbol::Gte) => Some(BinaryOp::Gte),

            TokenKind::Symbol(Symbol::Plus) => Some(BinaryOp::Add),
            TokenKind::Symbol(Symbol::Minus) => Some(BinaryOp::Sub),
            TokenKind::Symbol(Symbol::Star) => Some(BinaryOp::Mul),
            TokenKind::Symbol(Symbol::Slash) => Some(BinaryOp::Div),
            TokenKind::Symbol(Symbol::Percent) => Some(BinaryOp::Rem),

            _ => None,
        }
    }

    fn binary_binding_power(op: BinaryOp) -> (u8, u8) {
        match op {
            BinaryOp::Assign => right_assoc(BP_ASSIGN),
            BinaryOp::Pipe => left_assoc(BP_PIPE),
            BinaryOp::Or => left_assoc(BP_OR),
            BinaryOp::And => left_assoc(BP_AND),
            BinaryOp::Eq | BinaryOp::NotEq => left_assoc(BP_EQUALITY),
            BinaryOp::Lt | BinaryOp::Lte | BinaryOp::Gt | BinaryOp::Gte => {
                left_assoc(BP_COMPARISON)
            }
            BinaryOp::Add | BinaryOp::Sub => left_assoc(BP_SUM),
            BinaryOp::Mul | BinaryOp::Div | BinaryOp::Rem => left_assoc(BP_PRODUCT),
        }
    }
}
