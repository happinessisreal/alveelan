#[derive(Debug, Clone, PartialEq)]
pub enum AlvType {
    Songkhya,    // i64
    Doshomik,    // f64
    Lekha,       // String
    SottoMittha, // bool
    Array(Box<AlvType>),
    Void,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    BoolLiteral(bool),
    Variable(String),
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
    Unary(UnaryOp, Box<Expr>),
    Call(String, Vec<Expr>),
    ArrayLiteral(Vec<Expr>),
    IndexAccess(Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let {
        name: String,
        alv_type: AlvType,
        value: Expr,
        is_mutable: bool,
    },
    Assignment {
        name: String,
        value: Expr,
    },
    IndexAssignment {
        target: Box<Expr>, // IndexAccess
        value: Expr,
    },
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    Return(Option<Expr>),
    Expression(Expr),
    Print(Expr), // দেখাও
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub alv_type: AlvType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: AlvType,
    pub body: Vec<Stmt>,
}
