use crate::{
    ast::{NodeId, NodeKind, Param, TableItem},
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

            if !self.take_if(Symbol::Comma)? {
                break;
            }
        }

        let end = self.expect_kind(Symbol::RBrace)?;

        Ok(self.ast.add(NodeKind::Table(items), open_span.merge(end)))
    }

    fn parse_table_key(&mut self, open_span: Span, row_index: usize) -> Result<NodeId, LangError> {
        if self.take_if(Symbol::LBracket)? {
            let key = self.parse_expression()?;
            self.expect_kind(Symbol::RBracket)?;
            self.expect_kind(Symbol::Eq)?;
            return Ok(key);
        }

        if self.take_if(Symbol::Dot)? {
            let token = self.take()?;
            if let TokenKind::Identifier(name) = token.kind {
                self.expect_kind(Symbol::Eq)?;
                return Ok(self.ast.add(NodeKind::String(name), token.span));
            } else {
                return Err(LangError::expected("identifier", &token.kind, token.span));
            }
        }

        Ok(self.ast.add(NodeKind::Number(row_index as f64), open_span))
    }

    pub(super) fn parse_function(&mut self, open_span: Span) -> Result<NodeId, LangError> {
        self.expect_kind(Symbol::LParen)?;

        let mut params: Vec<Param> = Vec::new();
        let mut saw_default = false;

        if !self.peek_kind(Symbol::RParen)? {
            loop {
                let tok = self.take()?;
                let name = match tok.kind {
                    TokenKind::Identifier(n) => n,
                    other => {
                        return Err(LangError::expected("identifier", &other, tok.span));
                    }
                };

                let default = if self.take_if(Symbol::Eq)? {
                    saw_default = true;
                    Some(self.parse_expression()?)
                } else {
                    if saw_default {
                        return Err(LangError::new(
                            "syntax error",
                            format!(
                                "required parameter '{}' cannot follow a parameter with a default",
                                name
                            ),
                            tok.span,
                        ));
                    }
                    None
                };

                params.push(Param { name, default });

                if !self.take_if(Symbol::Comma)? {
                    break;
                }
            }
        }

        self.expect_kind(Symbol::RParen)?;

        let body = self.parse_block(&[TokenKind::Keyword(Keyword::End)])?;
        let end = self.expect_kind(Keyword::End)?;

        Ok(self
            .ast
            .add(NodeKind::Function { params, body }, open_span.merge(end)))
    }
}
