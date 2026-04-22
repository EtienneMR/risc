//! Parser top level: token stream access and block parsing.
//! Owns the Lexer, a one-token lookahead slot, and the Ast arena being built.
//! parse_block() collects expressions until a terminator token is seen.
//! parse_call / parse_property / parse_index handle postfix syntax.
//! Expression, statement, and compound forms are split across sub-modules.

use crate::{
    ast::{Ast, CallArg, NodeId, NodeKind, Program},
    error::LangError,
    lexer::{Lexer, Symbol, Token, TokenKind},
    source::Span,
};

mod compound;
mod expr;
mod stmt;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    lookahead: Option<Token>,
    ast: Ast,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            lexer,
            lookahead: None,
            ast: Ast::new(),
        }
    }

    pub fn parse(mut self) -> Result<Program, LangError> {
        let roots = self.parse_nodes(&[TokenKind::EndOfFile])?;
        Ok(Program {
            ast: self.ast,
            roots,
        })
    }

    fn take(&mut self) -> Result<Token, LangError> {
        match self.lookahead.take() {
            Some(tok) => Ok(tok),
            None => self.lexer.next_token(),
        }
    }

    fn take_if(&mut self, kind: impl Into<TokenKind>) -> Result<bool, LangError> {
        if self.peek_kind(kind)? {
            self.take()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn peek(&mut self) -> Result<&Token, LangError> {
        if self.lookahead.is_none() {
            self.lookahead = Some(self.take()?);
        }

        Ok(self
            .lookahead
            .as_ref()
            .expect("lookahead should be defined"))
    }

    fn peek_kind(&mut self, kind: impl Into<TokenKind>) -> Result<bool, LangError> {
        Ok(self.peek()?.kind == kind.into())
    }

    fn expect_kind(&mut self, kind: impl Into<TokenKind>) -> Result<Span, LangError> {
        let expected = kind.into();
        let token = self.take()?;

        if token.kind == expected {
            Ok(token.span)
        } else {
            Err(LangError::expected(expected, &token.kind, token.span))
        }
    }

    fn parse_block(&mut self, terminators: &[TokenKind]) -> Result<NodeId, LangError> {
        let nodes = self.parse_nodes(terminators)?;

        let span = if let Some(last) = nodes.last() {
            self.ast.get(*last).span.merge(self.ast.get(nodes[0]).span)
        } else {
            Span::INTERNAL
        };

        Ok(self.ast.add(NodeKind::Block { nodes }, span))
    }

    fn parse_nodes(&mut self, terminators: &[TokenKind]) -> Result<Vec<NodeId>, LangError> {
        let mut nodes = Vec::new();

        loop {
            let kind = &self.peek()?.kind;
            if terminators.iter().any(|t| t == kind) {
                break;
            }
            nodes.push(self.parse_expression()?);
        }

        Ok(nodes)
    }

    fn parse_call(&mut self, callee: NodeId) -> Result<NodeId, LangError> {
        let mut args: Vec<CallArg> = Vec::new();
        let mut saw_named = false;

        if !self.peek_kind(Symbol::RParen)? {
            loop {
                if self.take_if(Symbol::Dot)? {
                    saw_named = true;

                    let tok = self.take()?;
                    let name = match tok.kind {
                        TokenKind::Identifier(n) => n,
                        other => {
                            return Err(LangError::expected("identifier", &other, tok.span));
                        }
                    };
                    self.expect_kind(Symbol::Eq)?;
                    let value = self.parse_expression()?;
                    args.push(CallArg {
                        name: Some(name),
                        value,
                    });
                } else {
                    if saw_named {
                        let span = self.peek()?.span;
                        return Err(LangError::new(
                            "syntax error",
                            "positional argument cannot follow a named argument".to_string(),
                            span,
                        ));
                    }
                    let value = self.parse_expression()?;
                    args.push(CallArg { name: None, value });
                }

                if !self.take_if(Symbol::Comma)? {
                    break;
                }
            }
        }

        let close = self.expect_kind(Symbol::RParen)?;
        let span = self.ast.get(callee).span.merge(close);

        Ok(self.ast.add(NodeKind::Call { callee, args }, span))
    }

    fn parse_property(&mut self, object: NodeId) -> Result<NodeId, LangError> {
        let name_tok = self.take()?;
        let property = match name_tok.kind {
            TokenKind::Identifier(n) => n,
            _ => {
                return Err(LangError::expected(
                    "identifier",
                    &name_tok.kind,
                    name_tok.span,
                ));
            }
        };

        let span = self.ast.get(object).span.merge(name_tok.span);
        let key = self.ast.add(NodeKind::String(property), name_tok.span);

        Ok(self.ast.add(NodeKind::Index { object, key }, span))
    }

    fn parse_index(&mut self, object: NodeId) -> Result<NodeId, LangError> {
        let key = self.parse_expression()?;
        let close = self.expect_kind(Symbol::RBracket)?;
        let span = self.ast.get(object).span.merge(close);

        Ok(self.ast.add(NodeKind::Index { object, key }, span))
    }
}
