use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Keywords
    Phangshon,  // ফাংশন
    Dhori,      // ধরি
    Jodi,       // যদি
    Nahle,      // নাহলে
    Jotokhkhon, // যতক্ষণ
    Ferat,      // ফেরত
    Sotto,      // সত্য
    Mittha,     // মিথ্যা
    Ebong,      // এবং
    Othoba,     // অথবা
    Na,         // না
    Dekhao,     // দেখাও
    Nao,        // নাও
    Shuru,      // শুরু
    Talika,     // তালিকা

    // Types
    Songkhya,    // সংখ্যা
    Doshomik,    // দশমিক
    Lekha,       // লেখা
    SottoMittha, // সত্যমিথ্যা

    // Literals
    Identifier(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),

    // Operators
    Plus,         // +
    Minus,        // -
    Star,         // *
    Slash,        // /
    Percent,      // %
    Equal,        // =
    EqualEqual,   // ==
    NotEqual,     // !=
    Less,         // <
    LessEqual,    // <=
    Greater,      // >
    GreaterEqual, // >=

    // Delimiters
    LParen,   // (
    RParen,   // )
    LBrace,   // {
    RBrace,   // }
    Comma,    // ,
    Colon,    // :
    Arrow,    // ->
    LBracket, // [
    RBracket, // ]

    // Control
    Eof,
    Unknown(char),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize, column: usize) -> Self {
        Self {
            token_type,
            lexeme,
            line,
            column,
        }
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::Phangshon => write!(f, "ফাংশন"),
            TokenType::Dhori => write!(f, "ধরি"),
            TokenType::Jodi => write!(f, "যদি"),
            TokenType::Nahle => write!(f, "নাহলে"),
            TokenType::Jotokhkhon => write!(f, "যতক্ষণ"),
            TokenType::Ferat => write!(f, "ফেরত"),
            TokenType::Sotto => write!(f, "সত্য"),
            TokenType::Mittha => write!(f, "মিথ্যা"),
            TokenType::Ebong => write!(f, "এবং"),
            TokenType::Othoba => write!(f, "অথবা"),
            TokenType::Na => write!(f, "না"),
            TokenType::Dekhao => write!(f, "দেখাও"),
            TokenType::Nao => write!(f, "নাও"),
            TokenType::Shuru => write!(f, "শুরু"),
            TokenType::Talika => write!(f, "তালিকা"),
            TokenType::Songkhya => write!(f, "সংখ্যা"),
            TokenType::Doshomik => write!(f, "দশমিক"),
            TokenType::Lekha => write!(f, "লেখা"),
            TokenType::SottoMittha => write!(f, "সত্যমিথ্যা"),
            TokenType::Identifier(s) => write!(f, "Identifier({})", s),
            TokenType::IntLiteral(n) => write!(f, "IntLiteral({})", n),
            TokenType::FloatLiteral(n) => write!(f, "FloatLiteral({})", n),
            TokenType::StringLiteral(s) => write!(f, "StringLiteral({})", s),
            TokenType::Plus => write!(f, "+"),
            TokenType::Minus => write!(f, "-"),
            TokenType::Star => write!(f, "*"),
            TokenType::Slash => write!(f, "/"),
            TokenType::Percent => write!(f, "%"),
            TokenType::Equal => write!(f, "="),
            TokenType::EqualEqual => write!(f, "=="),
            TokenType::NotEqual => write!(f, "!="),
            TokenType::Less => write!(f, "<"),
            TokenType::LessEqual => write!(f, "<="),
            TokenType::Greater => write!(f, ">"),
            TokenType::GreaterEqual => write!(f, ">="),
            TokenType::LParen => write!(f, "("),
            TokenType::RParen => write!(f, ")"),
            TokenType::LBrace => write!(f, "{{"),
            TokenType::RBrace => write!(f, "}}"),
            TokenType::Comma => write!(f, ","),
            TokenType::Colon => write!(f, ":"),
            TokenType::Arrow => write!(f, "->"),
            TokenType::LBracket => write!(f, "["),
            TokenType::RBracket => write!(f, "]"),
            TokenType::Eof => write!(f, "EOF"),
            TokenType::Unknown(c) => write!(f, "Unknown({})", c),
        }
    }
}
