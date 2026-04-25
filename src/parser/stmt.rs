//! Parsers for compound statements: if/then/elseif/else/end, for/in/do/end, while/do/end.
//! Also handles try/catch <kind> as <binding>/else/end and the declaration (let / let fn) form.
//! Each parser is called after the opening keyword token has been consumed by the caller.
//! All branches parse sub-blocks with explicit terminator token sets to handle nested forms.
//! Declaration and function-literal parsing live in compound.rs and expr.rs respectively.

use crate::{
    ast::{CatchArm, NodeId, NodeKind},
    error::LangError,
    lexer::{Keyword, Symbol, Token, TokenKind},
    source::Span,
};

impl<'a> super::Parser<'a> {
    pub(super) fn parse_if(&mut self, open_span: Span) -> Result<NodeId, LangError> {
        let condition = self.parse_expression()?;
        let body_open_span = self.expect_kind(Keyword::Then)?;
        let then_branch = self.parse_block(
            body_open_span,
            &[
                TokenKind::Keyword(Keyword::ElseIf),
                TokenKind::Keyword(Keyword::Else),
                TokenKind::Keyword(Keyword::End),
            ],
        )?;

        let (else_branch, end) = if let Some(token) = self.take_if(Keyword::ElseIf)? {
            let nested_if = self.parse_if(token.span)?;
            (Some(nested_if), self.ast.get(nested_if).span)
        } else {
            let else_branch = if let Some(token) = self.take_if(Keyword::Else)? {
                Some(self.parse_block(token.span, &[TokenKind::Keyword(Keyword::End)])?)
            } else {
                None
            };
            let end = self.expect_kind(Keyword::End)?;
            (else_branch, end)
        };

        Ok(self.ast.add(
            NodeKind::If {
                condition,
                then_branch,
                else_branch,
            },
            open_span.merge(end),
        ))
    }

    pub(super) fn parse_for(&mut self, open_span: Span) -> Result<NodeId, LangError> {
        let (name, _) = self.expect_identifier()?;

        self.expect_kind(Keyword::In)?;
        let iterator = self.parse_expression()?;
        let body_open_span = self.expect_kind(Keyword::Do)?;
        let body = self.parse_block(body_open_span, &[TokenKind::Keyword(Keyword::End)])?;

        let end = self.expect_kind(Keyword::End)?;

        Ok(self.ast.add(
            NodeKind::For {
                identifier: name,
                iterator,
                body,
            },
            open_span.merge(end),
        ))
    }

    pub(super) fn parse_while(&mut self, open_span: Span) -> Result<NodeId, LangError> {
        let condition = self.parse_expression()?;
        let body_open_span = self.expect_kind(Keyword::Do)?;
        let body = self.parse_block(body_open_span, &[TokenKind::Keyword(Keyword::End)])?;

        let end = self.expect_kind(Keyword::End)?;

        Ok(self
            .ast
            .add(NodeKind::While { condition, body }, open_span.merge(end)))
    }

    pub(super) fn parse_declaration(&mut self, open_span: Span) -> Result<NodeId, LangError> {
        let is_function = self.take_if(Keyword::Fn)?.is_some();

        let identifier_token = self.take()?;
        let Token {
            kind: TokenKind::Identifier(identifier),
            ..
        } = identifier_token
        else {
            return Err(LangError::expected_token(
                "identifier",
                &identifier_token.kind,
                identifier_token.span,
            ));
        };

        let value = if is_function {
            self.parse_function(open_span)?
        } else {
            self.expect_kind(Symbol::Eq)?;
            self.parse_expression()?
        };

        let span = open_span.merge(self.ast.get(value).span);
        Ok(self
            .ast
            .add(NodeKind::Declaration { identifier, value }, span))
    }

    pub(super) fn parse_try_catch(&mut self, open_span: Span) -> Result<NodeId, LangError> {
        let body = self.parse_block(
            open_span,
            &[
                TokenKind::Keyword(Keyword::Catch),
                TokenKind::Keyword(Keyword::Else),
                TokenKind::Keyword(Keyword::End),
            ],
        )?;

        let mut catches = Vec::new();

        while let Some(token) = self.take_if(Keyword::Catch)? {
            catches.push(self.parse_catch_arm(token.span)?);
        }

        let else_branch = if let Some(token) = self.take_if(Keyword::Else)? {
            Some(self.parse_block(token.span, &[TokenKind::Keyword(Keyword::End)])?)
        } else {
            None
        };

        let end = self.expect_kind(Keyword::End)?;

        Ok(self.ast.add(
            NodeKind::TryCatch {
                body,
                catches,
                else_branch,
            },
            open_span.merge(end),
        ))
    }

    fn parse_catch_arm(&mut self, open_span: Span) -> Result<CatchArm, LangError> {
        let kind_filter = if self.peek_kind(Keyword::As)? {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.expect_kind(Keyword::As)?;

        let binding_tok = self.take()?;
        let binding = match binding_tok.kind {
            TokenKind::Identifier(n) => n,
            _ => {
                return Err(LangError::expected_token(
                    "identifier",
                    &binding_tok.kind,
                    binding_tok.span,
                ));
            }
        };

        let body = self.parse_block(
            open_span,
            &[
                TokenKind::Keyword(Keyword::Catch),
                TokenKind::Keyword(Keyword::Else),
                TokenKind::Keyword(Keyword::End),
            ],
        )?;

        Ok(CatchArm {
            kind_filter,
            binding,
            body,
        })
    }
}
