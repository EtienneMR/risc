use crate::source::Span;

#[derive(Debug, Clone)]
pub struct Block {
    pub exprs: Vec<Expr>,
}

impl Block {
    pub fn new() -> Self {
        Self { exprs: Vec::new() }
    }
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

impl Expr {
    pub fn new(kind: ExprKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,

    Identifier(String),

    Unary {
        op: UnaryOp,
        rhs: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },

    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Index {
        object: Box<Expr>,
        key: Box<Expr>,
    },

    If {
        condition: Box<Expr>,
        then_branch: Block,
        else_branch: Option<Block>,
    },
    For {
        identifier: String,
        iterator: Box<Expr>,
        body: Block,
    },
    While {
        condition: Box<Expr>,
        body: Block,
    },

    TryCatch {
        body: Block,
        catches: Vec<CatchArm>,
        else_branch: Option<Block>,
    },

    Table(Vec<TableRow>),

    Bind {
        identifier: String,
        value: Box<Expr>,
    },

    Function {
        params: Vec<String>,
        body: Block,
    },

    Break,
    Continue,
    Return(Box<Expr>),
}

#[derive(Debug, Clone)]
pub struct CatchArm {
    pub kind_filter: Option<Box<Expr>>,
    pub binding: String,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct TableRow {
    pub key: Box<Expr>,
    pub value: Box<Expr>,
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
