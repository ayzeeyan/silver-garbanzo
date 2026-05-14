#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Nil,
    Bool(bool),
    Number(f64),
    String(Vec<u8>),
    Ident(String),
    Table(Vec<(Option<Expr>, Expr)>),
    Index(Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    MethodCall(Box<Expr>, String, Vec<Expr>),
    BinOp(String, Box<Expr>, Box<Expr>),
    UnOp(String, Box<Expr>),
    Length(Box<Expr>),
}

#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    LocalDecl(Vec<String>, Vec<Expr>),
    Assign(Vec<Expr>, Vec<Expr>),
    FunctionDecl(String, Vec<String>, Box<Block>),
    LocalFunctionDecl(String, Vec<String>, Box<Block>),
    While(Expr, Box<Block>),
    Repeat(Box<Block>, Expr),
    If(Vec<(Expr, Block)>, Option<Block>),
    ForNum(String, Expr, Expr, Option<Expr>, Box<Block>),
    Break,
    CallStmt(Expr),
    Return(Vec<Expr>),
}

#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq)]
pub struct Block(pub Vec<Stmt>);
