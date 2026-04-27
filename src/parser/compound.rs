//! Parsers for table literals, function definitions, and function/call argument lists.
//! Table keys: [expr]=value, .name=value, or implicit 0-based sequential integer.
//! Function parameters support default values; a required param after a default is a syntax error.
//! Named call arguments use .name=value syntax; a positional arg after a named arg is an error.
//! These parsers are invoked after the opening token is already consumed by the calling rule.

use crate::{
    ast::{NodeId, NodeKind, Param, ParamKind, TableItem},
    error::LangError,
    lexer::{Keyword, Symbol, TokenKind},
    source::Span,
};

impl<'a> super::Parser<'a> {
    pub(super) fn parse_table(&mut self, open_span: Span) -> Result<NodeId, LangError> {
        let mut items: Vec<TableItem> = Vec::new();

        while !self.peek_kind(Symbol::RBrace)? {
            let key = self.parse_table_key(open_span, items.len())?;
            let value = self.parse_expression()?;

            items.push(TableItem { key, value });

            if self.take_if(Symbol::Comma)?.is_none() {
                break;
            }
        }

        let end = self.expect_kind(Symbol::RBrace)?;

        Ok(self.ast.add(NodeKind::Table(items), open_span.merge(end)))
    }

    fn parse_table_key(&mut self, open_span: Span, row_index: usize) -> Result<NodeId, LangError> {
        if self.take_if(Symbol::LBracket)?.is_some() {
            let key = self.parse_expression()?;
            self.expect_kind(Symbol::RBracket)?;
            self.expect_kind(Symbol::Eq)?;
            return Ok(key);
        }

        if self.take_if(Symbol::Dot)?.is_some() {
            let token = self.take()?;
            if let TokenKind::Identifier(name) = token.kind {
                self.expect_kind(Symbol::Eq)?;
                return Ok(self.ast.add(NodeKind::String(name), token.span));
            } else {
                return Err(LangError::expected_token(
                    "identifier",
                    &token.kind,
                    token.span,
                ));
            }
        }

        Ok(self.ast.add(NodeKind::Number(row_index as f64), open_span))
    }

    pub(super) fn parse_function(&mut self, open_span: Span) -> Result<NodeId, LangError> {
        self.expect_kind(Symbol::LParen)?;

        let mut params: Vec<Param> = Vec::new();
        let mut saw_optional = false;

        if !self.peek_kind(Symbol::RParen)? {
            loop {
                if self.take_if(Symbol::DotDot)?.is_some() {
                    let name = self.expect_identifier()?.0;

                    params.push(Param {
                        name,
                        kind: ParamKind::Rest,
                    });
                    break;
                } else {
                    let (name, param_span) = self.expect_identifier()?;

                    if self.take_if(Symbol::Eq)?.is_some() {
                        saw_optional = true;
                        params.push(Param {
                            name,
                            kind: ParamKind::Optional(self.parse_expression()?),
                        });
                    } else if saw_optional {
                        return Err(LangError::invalid_syntax(
                            "required parameter after optional",
                            format!(
                                "required parameter '{name}' must appear before any default or rest parameter."
                            ),
                            param_span,
                        ));
                    } else {
                        params.push(Param {
                            name,
                            kind: ParamKind::Required,
                        });
                    }
                }

                if self.take_if(Symbol::Comma)?.is_none() {
                    break;
                }
            }
        }

        let body_open_span = self.expect_kind(Symbol::RParen)?;

        let body = self.parse_block(body_open_span, &[TokenKind::Keyword(Keyword::End)])?;

        let end = self.expect_kind(Keyword::End)?;

        Ok(self
            .ast
            .add(NodeKind::Function { params, body }, open_span.merge(end)))
    }
}
