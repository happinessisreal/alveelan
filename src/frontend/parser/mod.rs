pub mod ast;
use crate::frontend::lexer::Lexer;
use crate::frontend::lexer::token::{Token, TokenType};
use crate::frontend::parser::ast::*;

/// Alveelan Parser performs recursive descent parsing to generate an AST.
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
    peek_token: Token,
}

impl<'a> Parser<'a> {
    /// Creates a new Parser from a Lexer instance.
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let current_token = lexer.next_token();
        let peek_token = lexer.next_token();
        Self {
            lexer,
            current_token,
            peek_token,
        }
    }

    fn advance(&mut self) {
        self.current_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    fn expect(&mut self, token_type: TokenType) -> Result<Token, String> {
        if std::mem::discriminant(&self.current_token.token_type)
            == std::mem::discriminant(&token_type)
        {
            let tok = self.current_token.clone();
            self.advance();
            Ok(tok)
        } else {
            Err(format!(
                "ত্রুটি [লাইন {}]: '{}' প্রত্যাশিত ছিল, কিন্তু '{}' পাওয়া গেছে।",
                self.current_token.line, token_type, self.current_token.token_type
            ))
        }
    }

    fn current_type(&self) -> &TokenType {
        &self.current_token.token_type
    }

    pub fn parse_program(&mut self) -> Result<Vec<Function>, String> {
        let mut functions = Vec::new();
        while !matches!(self.current_type(), TokenType::Eof) {
            functions.push(self.parse_function()?);
        }
        Ok(functions)
    }

    fn parse_function(&mut self) -> Result<Function, String> {
        self.expect(TokenType::Phangshon)?;

        let name = match self.current_type() {
            TokenType::Identifier(s) => {
                let name = s.clone();
                self.advance();
                name
            }
            TokenType::Shuru => {
                self.advance();
                "শুরু".to_string()
            }
            _ => {
                return Err(format!(
                    "ত্রুটি [লাইন {}]: ফাংশনের একটি নাম প্রত্যাশিত ছিল, কিন্তু '{}' পাওয়া গেছে।",
                    self.current_token.line, self.current_token.token_type
                ));
            }
        };

        self.expect(TokenType::LParen)?;
        let mut params = Vec::new();
        if !matches!(self.current_type(), TokenType::RParen) {
            loop {
                let param_name_token = self.expect(TokenType::Identifier("".to_string()))?;
                let param_name = match param_name_token.token_type {
                    TokenType::Identifier(s) => s,
                    _ => unreachable!(),
                };

                self.expect(TokenType::Colon)?;
                let alv_type = self.parse_type()?;

                params.push(Parameter {
                    name: param_name,
                    alv_type,
                });

                if matches!(self.current_type(), TokenType::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect(TokenType::RParen)?;

        let mut return_type = AlvType::Void;
        if matches!(self.current_type(), TokenType::Arrow) {
            self.advance();
            return_type = self.parse_type()?;
        }

        let body = self.parse_block()?;

        Ok(Function {
            name,
            params,
            return_type,
            body,
        })
    }

    fn parse_type(&mut self) -> Result<AlvType, String> {
        match self.current_type() {
            TokenType::Songkhya => {
                self.advance();
                Ok(AlvType::Songkhya)
            }
            TokenType::Doshomik => {
                self.advance();
                Ok(AlvType::Doshomik)
            }
            TokenType::Lekha => {
                self.advance();
                Ok(AlvType::Lekha)
            }
            TokenType::SottoMittha => {
                self.advance();
                Ok(AlvType::SottoMittha)
            }
            TokenType::Talika => {
                self.advance();
                self.expect(TokenType::LBracket)?;
                let inner = self.parse_type()?;
                self.expect(TokenType::RBracket)?;
                Ok(AlvType::Array(Box::new(inner)))
            }
            _ => Err(format!(
                "ত্রুটি [লাইন {}]: একটি সঠিক ধরন (যেমন: সংখ্যা, লেখা) প্রত্যাশিত ছিল।",
                self.current_token.line
            )),
        }
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        self.expect(TokenType::LBrace)?;
        let mut statements = Vec::new();
        while !matches!(self.current_type(), TokenType::RBrace) && !matches!(self.current_type(), TokenType::Eof) {
            statements.push(self.parse_statement()?);
        }
        self.expect(TokenType::RBrace)?;
        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<Stmt, String> {
        match self.current_type() {
            TokenType::Dhori => self.parse_let_statement(),
            TokenType::Jodi => self.parse_if_statement(),
            TokenType::Jotokhkhon => self.parse_while_statement(),
            TokenType::Ferat => self.parse_return_statement(),
            TokenType::Dekhao => self.parse_print_statement(),
            _ => {
                let expr = self.parse_expression(0)?;
                if matches!(self.current_type(), TokenType::Equal) {
                    self.advance();
                    let value = self.parse_expression(0)?;
                    match expr {
                        Expr::Variable(name) => Ok(Stmt::Assignment { name, value }),
                        Expr::IndexAccess(target, index) => Ok(Stmt::IndexAssignment {
                            target: Box::new(Expr::IndexAccess(target, index)),
                            value,
                        }),
                        _ => Err("ত্রুটি: বামপাশে একটি চলক বা ইনডেক্স থাকতে হবে।".to_string()),
                    }
                } else {
                    Ok(Stmt::Expression(expr))
                }
            }
        }
    }

    fn parse_let_statement(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume ধরি
        let name_token = self.expect(TokenType::Identifier("".to_string()))?;
        let name = match name_token.token_type {
            TokenType::Identifier(s) => s,
            _ => unreachable!(),
        };

        self.expect(TokenType::Colon)?;
        let alv_type = self.parse_type()?;

        self.expect(TokenType::Equal)?;
        let value = self.parse_expression(0)?;

        Ok(Stmt::Let {
            name,
            alv_type,
            value,
            is_mutable: true, // simplified for kids
        })
    }

    fn parse_if_statement(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume যদি
        self.expect(TokenType::LParen)?;
        let condition = self.parse_expression(0)?;
        self.expect(TokenType::RParen)?;

        let then_branch = self.parse_block()?;
        let mut else_branch = None;

        if matches!(self.current_type(), TokenType::Nahle) {
            self.advance();
            else_branch = Some(self.parse_block()?);
        }

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn parse_while_statement(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume ততক্ষণ
        self.expect(TokenType::LParen)?;
        let condition = self.parse_expression(0)?;
        self.expect(TokenType::RParen)?;

        let body = self.parse_block()?;

        Ok(Stmt::While { condition, body })
    }

    fn parse_return_statement(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume ফেরত
        let mut value = None;
        if !matches!(self.current_type(), TokenType::RBrace) && !matches!(self.current_type(), TokenType::Eof) {
            // Check if there is an expression to return (simplified)
            // If it's not a delimiter, assume it's an expression
            if !matches!(self.current_type(), TokenType::RBrace) {
                value = Some(self.parse_expression(0)?);
            }
        }
        Ok(Stmt::Return(value))
    }

    fn parse_print_statement(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume দেখাও
        self.expect(TokenType::LParen)?;
        let expr = self.parse_expression(0)?;
        self.expect(TokenType::RParen)?;
        Ok(Stmt::Print(expr))
    }

    // Expression Parsing (Pratt Parsing inspired)
    fn parse_expression(&mut self, precedence: i8) -> Result<Expr, String> {
        let mut left = self.parse_primary()?;

        while precedence < self.current_precedence() {
            let op_type = self.current_token.token_type.clone();
            self.advance();
            let op = match op_type {
                TokenType::Plus => BinaryOp::Add,
                TokenType::Minus => BinaryOp::Sub,
                TokenType::Star => BinaryOp::Mul,
                TokenType::Slash => BinaryOp::Div,
                TokenType::Percent => BinaryOp::Mod,
                TokenType::EqualEqual => BinaryOp::Eq,
                TokenType::NotEqual => BinaryOp::Ne,
                TokenType::Less => BinaryOp::Lt,
                TokenType::LessEqual => BinaryOp::Le,
                TokenType::Greater => BinaryOp::Gt,
                TokenType::GreaterEqual => BinaryOp::Ge,
                TokenType::Ebong => BinaryOp::And,
                TokenType::Othoba => BinaryOp::Or,
                _ => break,
            };

            let next_precedence = self.precedence(&op_type);
            let right = self.parse_expression(next_precedence)?;
            left = Expr::Binary(op, Box::new(left), Box::new(right));
        }

        // Postfix IndexAccess
        while matches!(self.current_type(), TokenType::LBracket) {
            self.advance();
            let index = self.parse_expression(0)?;
            self.expect(TokenType::RBracket)?;
            left = Expr::IndexAccess(Box::new(left), Box::new(index));
        }

        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.current_token.token_type.clone() {
            TokenType::IntLiteral(n) => {
                self.advance();
                Ok(Expr::IntLiteral(n))
            }
            TokenType::FloatLiteral(n) => {
                self.advance();
                Ok(Expr::FloatLiteral(n))
            }
            TokenType::StringLiteral(s) => {
                self.advance();
                Ok(Expr::StringLiteral(s))
            }
            TokenType::Sotto => {
                self.advance();
                Ok(Expr::BoolLiteral(true))
            }
            TokenType::Mittha => {
                self.advance();
                Ok(Expr::BoolLiteral(false))
            }
            TokenType::Identifier(s) => {
                self.advance();
                if matches!(self.current_type(), TokenType::LParen) {
                    self.advance(); // consume (
                    let mut args = Vec::new();
                    if !matches!(self.current_type(), TokenType::RParen) {
                        loop {
                            args.push(self.parse_expression(0)?);
                            if matches!(self.current_type(), TokenType::Comma) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    self.expect(TokenType::RParen)?;
                    Ok(Expr::Call(s, args))
                } else {
                    Ok(Expr::Variable(s))
                }
            }
            TokenType::LParen => {
                self.advance();
                let expr = self.parse_expression(0)?;
                self.expect(TokenType::RParen)?;
                Ok(expr)
            }
            TokenType::Na | TokenType::Minus => {
                let op_type = self.current_token.token_type.clone();
                self.advance();
                let op = if matches!(op_type, TokenType::Na) {
                    UnaryOp::Not
                } else {
                    UnaryOp::Neg
                };
                let right = self.parse_expression(7)?; // Unary precedence
                Ok(Expr::Unary(op, Box::new(right)))
            }
            TokenType::LBracket => {
                self.advance();
                let mut elements = Vec::new();
                if !matches!(self.current_type(), TokenType::RBracket) {
                    loop {
                        elements.push(self.parse_expression(0)?);
                        if matches!(self.current_type(), TokenType::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                self.expect(TokenType::RBracket)?;
                Ok(Expr::ArrayLiteral(elements))
            }
            _ => Err(format!(
                "ত্রুটি [লাইন {}]: একটি সঠিক এক্সপ্রেশন প্রত্যাশিত ছিল, কিন্তু '{}' পাওয়া গেছে।",
                self.current_token.line, self.current_token.token_type
            )),
        }
    }

    fn current_precedence(&self) -> i8 {
        self.precedence(self.current_type())
    }

    fn precedence(&self, token_type: &TokenType) -> i8 {
        match token_type {
            TokenType::Othoba => 1,
            TokenType::Ebong => 2,
            TokenType::EqualEqual | TokenType::NotEqual => 3,
            TokenType::Less
            | TokenType::LessEqual
            | TokenType::Greater
            | TokenType::GreaterEqual => 4,
            TokenType::Plus | TokenType::Minus => 5,
            TokenType::Star | TokenType::Slash | TokenType::Percent => 6,
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::lexer::Lexer;

    #[test]
    fn test_parse_simple_function() {
        let input = "ফাংশন শুরু() { দেখাও(১০) }";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.len(), 1);
        assert_eq!(program[0].name, "শুরু");
        assert_eq!(program[0].body.len(), 1);

        match &program[0].body[0] {
            Stmt::Print(Expr::IntLiteral(n)) => assert_eq!(*n, 10),
            _ => panic!(
                "Expected Print(IntLiteral(10)), got {:?}",
                program[0].body[0]
            ),
        }
    }

    #[test]
    fn test_parse_let_and_expression() {
        let input = "
        ফাংশন যোগফল() {
            ধরি ক: সংখ্যা = ১০
            ধরি খ: সংখ্যা = ২০
            দেখাও(ক + খ)
        }
        ";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.len(), 1);
        let body = &program[0].body;
        assert_eq!(body.len(), 3);

        match &body[0] {
            Stmt::Let { name, alv_type, .. } => {
                assert_eq!(name, "ক");
                assert_eq!(alv_type, &AlvType::Songkhya);
            }
            _ => panic!("Expected Let, got {:?}", body[0]),
        }

        match &body[2] {
            Stmt::Print(Expr::Binary(BinaryOp::Add, _, _)) => {}
            _ => panic!("Expected Print(Binary(Add, ...)), got {:?}", body[2]),
        }
    }

    #[test]
    fn test_parse_if_else() {
        let input = "
        ফাংশন তুলনা(ক: সংখ্যা) {
            যদি (ক > ১০) {
                দেখাও(১)
            } নাহলে {
                দেখাও(০)
            }
        }
        ";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.len(), 1);
        match &program[0].body[0] {
            Stmt::If { else_branch, .. } => {
                assert!(else_branch.is_some());
            }
            _ => panic!("Expected If-Else, got {:?}", program[0].body[0]),
        }
    }

    #[test]
    fn test_parse_array() {
        let input = "
        ফাংশন উদাহরণ() {
            ধরি ক: তালিকা[সংখ্যা] = [১, ২, ৩]
            দেখাও(ক[০])
            ক[১] = ২০
        }
        ";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.len(), 1);
        let body = &program[0].body;
        assert_eq!(body.len(), 3);

        match &body[0] {
            Stmt::Let {
                alv_type, value, ..
            } => {
                assert!(matches!(alv_type, AlvType::Array(_)));
                assert!(matches!(value, Expr::ArrayLiteral(_)));
            }
            _ => panic!("Expected Let Array, got {:?}", body[0]),
        }

        match &body[1] {
            Stmt::Print(Expr::IndexAccess(_, _)) => {}
            _ => panic!("Expected Print IndexAccess, got {:?}", body[1]),
        }

        match &body[2] {
            Stmt::IndexAssignment { .. } => {}
            _ => panic!("Expected IndexAssignment, got {:?}", body[2]),
        }
    }
}
