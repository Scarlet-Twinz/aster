use crate::type_system::Type;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let { name: String, type_annotation: Option<Type>, initializer: Expr },
    Expression(Expr),
    Print(Expr),
    Return(Option<Expr>),
    Block(Vec<Stmt>),
    If { condition: Expr, then_branch: Box<Stmt>, else_branch: Option<Box<Stmt>> },
    Function { name: String, params: Vec<String>, body: Vec<Stmt> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    String(String),
    Boolean(bool),
    Variable(String),
    Unary { operator: UnaryOp, operand: Box<Expr> },
    Binary { left: Box<Expr>, operator: BinaryOp, right: Box<Expr> },
    Assign { name: String, value: Box<Expr> },
    Call { callee: Box<Expr>, arguments: Vec<Expr> },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp { Negate, Not }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add, Subtract, Multiply, Divide, Modulo,
    Equal, NotEqual, Less, LessEqual, Greater, GreaterEqual,
    And, Or,
}
