//! AST arena for the Risc language: every node is stored by NodeId (typed usize index).
//! NodeKind covers all expression and statement forms: literals, operators, control flow,
//! function definitions, table literals, pipe, calls, and variable declarations.
//! Param, CallArg, CatchArm, and TableItem carry the structured sub-data for each form.
//! Program bundles the Ast arena with the ordered root NodeIds produced by the parser.

use crate::source::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum NodeKind {
    Block {
        nodes: Vec<NodeId>,
    },

    Number(f64),
    String(String),
    Boolean(bool),
    Nil,

    Identifier(String),

    Unary {
        op: UnaryOp,
        right: NodeId,
    },

    Binary {
        op: BinaryOp,
        left: NodeId,
        right: NodeId,
    },

    Call {
        callee: NodeId,
        args: Vec<CallArg>,
        last_is_rest: bool,
    },

    Index {
        object: NodeId,
        key: NodeId,
    },

    If {
        condition: NodeId,
        then_branch: NodeId,
        else_branch: Option<NodeId>,
    },

    For {
        identifier: String,
        iterator: NodeId,
        body: NodeId,
    },

    While {
        condition: NodeId,
        body: NodeId,
    },

    TryCatch {
        body: NodeId,
        catches: Vec<CatchArm>,
        else_branch: Option<NodeId>,
    },

    Declaration {
        identifier: String,
        value: NodeId,
    },

    Function {
        params: Vec<Param>,
        body: NodeId,
    },

    Table(Vec<TableItem>),

    Break(NodeId),
    Continue,
    Return(NodeId),
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub kind: ParamKind,
}

#[derive(Debug, Clone)]
pub enum ParamKind {
    Required,
    Optional(NodeId),
    Rest,
}

#[derive(Debug, Clone)]
pub struct CallArg {
    pub name: Option<String>,
    pub value: NodeId,
}

#[derive(Debug, Clone)]
pub struct CatchArm {
    pub kind_filter: Option<NodeId>,
    pub binding: String,
    pub body: NodeId,
}

#[derive(Debug, Clone)]
pub struct TableItem {
    pub key: NodeId,
    pub value: NodeId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Assign,
    Pipe,
    Or,
    And,
    Eq,
    NotEq,
    Lt,
    Lte,
    Gt,
    Gte,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}

#[derive(Debug)]
pub struct Ast {
    nodes: Vec<Node>,
}

impl Ast {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn add(&mut self, kind: NodeKind, span: Span) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(Node { kind, span });
        id
    }

    pub fn get(&self, id: NodeId) -> &Node {
        &self.nodes[id.0 as usize]
    }
}

#[derive(Debug)]
pub struct Program {
    pub ast: Ast,
    pub roots: Vec<NodeId>,
}
