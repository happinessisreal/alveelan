use crate::frontend::parser::ast::*;
use std::collections::HashMap;

pub struct SymbolTable {
    variables: Vec<HashMap<String, AlvType>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            variables: vec![HashMap::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.variables.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.variables.pop();
    }

    pub fn define(&mut self, name: String, alv_type: AlvType) {
        if let Some(scope) = self.variables.last_mut() {
            scope.insert(name, alv_type);
        }
    }

    pub fn lookup(&self, name: &str) -> Option<AlvType> {
        for scope in self.variables.iter().rev() {
            if let Some(alv_type) = scope.get(name) {
                return Some(alv_type.clone());
            }
        }
        None
    }
}

/// SemanticAnalyzer handles type checking, scope management, and function signatures.
pub struct SemanticAnalyzer {
    symbol_table: SymbolTable,
    functions: HashMap<String, (Vec<Parameter>, AlvType)>,
}

impl SemanticAnalyzer {
    /// Creates a new SemanticAnalyzer instance.
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            functions: HashMap::new(),
        }
    }

    pub fn analyze(&mut self, program: &[Function]) -> Result<(), String> {
        // First pass: collect function signatures
        for func in program {
            if self.functions.contains_key(&func.name) {
                return Err(format!(
                    "ত্রুটি: '{}' নামের ফাংশনটি ইতিমধ্যে সংজ্ঞায়িত করা হয়েছে।",
                    func.name
                ));
            }
            self.functions.insert(
                func.name.clone(),
                (func.params.clone(), func.return_type.clone()),
            );
        }

        // Check if শুরু exists
        if !self.functions.contains_key("শুরু") {
            return Err("ত্রুটি: 'শুরু' নামের প্রধান ফাংশনটি পাওয়া যায়নি।".to_string());
        }

        // Second pass: analyze bodies
        for func in program {
            self.analyze_function(func)?;
        }

        Ok(())
    }

    fn analyze_function(&mut self, func: &Function) -> Result<(), String> {
        self.symbol_table.push_scope();

        for param in &func.params {
            self.symbol_table
                .define(param.name.clone(), param.alv_type.clone());
        }

        for stmt in &func.body {
            self.analyze_statement(stmt, &func.return_type)?;
        }

        self.symbol_table.pop_scope();
        Ok(())
    }

    fn analyze_statement(&mut self, stmt: &Stmt, return_type: &AlvType) -> Result<(), String> {
        match stmt {
            Stmt::Let {
                name,
                alv_type,
                value,
                ..
            } => {
                let value_type = self.analyze_expression(value)?;
                if !self.types_match(alv_type, &value_type) {
                    return Err(format!(
                        "ত্রুটি: '{}' চলকটির ধরন হল {}, কিন্তু আপনি একে {} করার চেষ্টা করছেন।",
                        name, alv_type, value_type
                    ));
                }
                self.symbol_table.define(name.clone(), alv_type.clone());
            }
            Stmt::Assignment { name, value } => {
                let var_type = self
                    .symbol_table
                    .lookup(name)
                    .ok_or(format!("ত্রুটি: '{}' নামের চলকটি পাওয়া যায়নি।", name))?;
                let value_type = self.analyze_expression(value)?;
                if !self.types_match(&var_type, &value_type) {
                    return Err(format!(
                        "ত্রুটি: '{}' চলকটির ধরন হল {}, কিন্তু আপনি এতে {} মান রাখার চেষ্টা করছেন।",
                        name, var_type, value_type
                    ));
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_type = self.analyze_expression(condition)?;
                if cond_type != AlvType::SottoMittha {
                    return Err("ত্রুটি: 'যদি' এর শর্তটি অবশ্যই সত্যমিথ্যা হতে হবে।".to_string());
                }

                self.symbol_table.push_scope();
                for s in then_branch {
                    self.analyze_statement(s, return_type)?;
                }
                self.symbol_table.pop_scope();

                if let Some(branch) = else_branch {
                    self.symbol_table.push_scope();
                    for s in branch {
                        self.analyze_statement(s, return_type)?;
                    }
                    self.symbol_table.pop_scope();
                }
            }
            Stmt::While { condition, body } => {
                let cond_type = self.analyze_expression(condition)?;
                if cond_type != AlvType::SottoMittha {
                    return Err("ত্রুটি: 'যতক্ষণ' এর শর্তটি অবশ্যই সত্যমিথ্যা হতে হবে।".to_string());
                }

                self.symbol_table.push_scope();
                for s in body {
                    self.analyze_statement(s, return_type)?;
                }
                self.symbol_table.pop_scope();
            }
            Stmt::Return(expr) => {
                let expr_type = if let Some(e) = expr {
                    self.analyze_expression(e)?
                } else {
                    AlvType::Void
                };
                if !self.types_match(return_type, &expr_type) {
                    return Err(format!(
                        "ত্রুটি: ফাংশনটি {} ফেরত দেবার কথা, কিন্তু আপনি {} ফেরত দিচ্ছেন।",
                        return_type, expr_type
                    ));
                }
            }
            Stmt::Expression(expr) => {
                self.analyze_expression(expr)?;
            }
            Stmt::Print(expr) => {
                self.analyze_expression(expr)?;
            }
            Stmt::IndexAssignment { target, value } => {
                let target_type = self.analyze_expression(target)?;
                let value_type = self.analyze_expression(value)?;
                if !self.types_match(&target_type, &value_type) {
                    return Err(format!(
                        "ত্রুটি: ইনডেক্সে {} মান রাখার চেষ্টা করছেন, কিন্তু ইনডেক্সটির ধরন হল {}",
                        value_type, target_type
                    ));
                }
            }
        }
        Ok(())
    }

    fn analyze_expression(&self, expr: &Expr) -> Result<AlvType, String> {
        match expr {
            Expr::IntLiteral(_) => Ok(AlvType::Songkhya),
            Expr::FloatLiteral(_) => Ok(AlvType::Doshomik),
            Expr::StringLiteral(_) => Ok(AlvType::Lekha),
            Expr::BoolLiteral(_) => Ok(AlvType::SottoMittha),
            Expr::Variable(name) => self
                .symbol_table
                .lookup(name)
                .ok_or(format!("ত্রুটি: '{}' নামের চলকটি পাওয়া যায়নি।", name)),
            Expr::Binary(op, left, right) => {
                let left_type = self.analyze_expression(left)?;
                let right_type = self.analyze_expression(right)?;

                match op {
                    BinaryOp::Add
                    | BinaryOp::Sub
                    | BinaryOp::Mul
                    | BinaryOp::Div
                    | BinaryOp::Mod => {
                        if left_type == AlvType::Songkhya && right_type == AlvType::Songkhya {
                            Ok(AlvType::Songkhya)
                        } else if left_type == AlvType::Doshomik && right_type == AlvType::Doshomik
                        {
                            Ok(AlvType::Doshomik)
                        } else {
                            Err("ত্রুটি: গাণিতিক কাজ শুধুমাত্র একই ধরণের সংখ্যার মধ্যে করা সম্ভব।".to_string())
                        }
                    }
                    BinaryOp::Eq
                    | BinaryOp::Ne
                    | BinaryOp::Lt
                    | BinaryOp::Le
                    | BinaryOp::Gt
                    | BinaryOp::Ge => {
                        if self.types_match(&left_type, &right_type) {
                            Ok(AlvType::SottoMittha)
                        } else {
                            Err("ত্রুটি: তুলনা শুধুমাত্র একই ধরণের মানের মধ্যে করা সম্ভব।".to_string())
                        }
                    }
                    BinaryOp::And | BinaryOp::Or => {
                        if left_type == AlvType::SottoMittha && right_type == AlvType::SottoMittha {
                            Ok(AlvType::SottoMittha)
                        } else {
                            Err("ত্রুটি: 'এবং/অথবা' শুধুমাত্র সত্যমিথ্যা মানের মধ্যে করা সম্ভব।".to_string())
                        }
                    }
                }
            }
            Expr::Unary(op, right) => {
                let right_type = self.analyze_expression(right)?;
                match op {
                    UnaryOp::Neg => {
                        if right_type == AlvType::Songkhya || right_type == AlvType::Doshomik {
                            Ok(right_type)
                        } else {
                            Err("ত্রুটি: নেতিবাচক চিহ্ন (-) শুধুমাত্র সংখ্যার আগে ব্যবহার করা সম্ভব।".to_string())
                        }
                    }
                    UnaryOp::Not => {
                        if right_type == AlvType::SottoMittha {
                            Ok(AlvType::SottoMittha)
                        } else {
                            Err("ত্রুটি: 'না' শুধুমাত্র সত্যমিথ্যা মানের সাথে ব্যবহার করা সম্ভব।".to_string())
                        }
                    }
                }
            }
            Expr::Call(name, args) => {
                let (params, return_type) = self
                    .functions
                    .get(name)
                    .ok_or(format!("ত্রুটি: '{}' নামের কোনো ফাংশন পাওয়া যায়নি।", name))?;

                if args.len() != params.len() {
                    return Err(format!(
                        "ত্রুটি: '{}' ফাংশনে {} টি মান প্রত্যাশিত ছিল, কিন্তু আপনি {} টি দিয়েছেন।",
                        name,
                        params.len(),
                        args.len()
                    ));
                }

                for (arg, param) in args.iter().zip(params.iter()) {
                    let arg_type = self.analyze_expression(arg)?;
                    if !self.types_match(&param.alv_type, &arg_type) {
                        return Err(format!(
                            "ত্রুটি: '{}' ফাংশনের '{}' প্যারামিটারটির জন্য {} প্রত্যাশিত ছিল।",
                            name, param.name, param.alv_type
                        ));
                    }
                }

                Ok(return_type.clone())
            }
            Expr::ArrayLiteral(elements) => {
                if elements.is_empty() {
                    return Err("ত্রুটি: খালি তালিকা (empty array) সমর্থন করা হয় না।".to_string());
                }
                let first_type = self.analyze_expression(&elements[0])?;
                for (i, elem) in elements.iter().enumerate().skip(1) {
                    let elem_type = self.analyze_expression(elem)?;
                    if !self.types_match(&first_type, &elem_type) {
                        return Err(format!(
                            "ত্রুটি: তালিকার ১ নং সদস্যের ধরন {}, কিন্তু {} নং সদস্যের ধরন {}। তালিকার সবগুলো সদস্য এক ধরনের হতে হবে।",
                            first_type,
                            crate::frontend::lexer::to_bangla_digits(i + 1),
                            elem_type
                        ));
                    }
                }
                Ok(AlvType::Array(Box::new(first_type)))
            }
            Expr::IndexAccess(target, index) => {
                let target_type = self.analyze_expression(target)?;
                let index_type = self.analyze_expression(index)?;

                if index_type != AlvType::Songkhya {
                    return Err("ত্রুটি: তালিকার ইনডেক্স অবশ্যই 'সংখ্যা' হতে হবে।".to_string());
                }

                match target_type {
                    AlvType::Array(inner) => Ok(*inner),
                    _ => Err("ত্রুটি: শুধুমাত্র তালিকাতে ইনডেক্স ব্যবহার করা সম্ভব।".to_string()),
                }
            }
        }
    }

    fn types_match(&self, expected: &AlvType, actual: &AlvType) -> bool {
        expected == actual
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_program() {
        let program = vec![Function {
            name: "শুরু".to_string(),
            params: vec![],
            return_type: AlvType::Void,
            body: vec![
                Stmt::Let {
                    name: "ক".to_string(),
                    alv_type: AlvType::Songkhya,
                    value: Expr::IntLiteral(10),
                    is_mutable: true,
                },
                Stmt::Print(Expr::Variable("ক".to_string())),
            ],
        }];

        let mut analyzer = SemanticAnalyzer::new();
        assert!(analyzer.analyze(&program).is_ok());
    }

    #[test]
    fn test_undefined_variable() {
        let program = vec![Function {
            name: "শুরু".to_string(),
            params: vec![],
            return_type: AlvType::Void,
            body: vec![Stmt::Print(Expr::Variable("খ".to_string()))],
        }];

        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&program);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("খ"));
    }

    #[test]
    fn test_type_mismatch() {
        let program = vec![Function {
            name: "শুরু".to_string(),
            params: vec![],
            return_type: AlvType::Void,
            body: vec![Stmt::Let {
                name: "ক".to_string(),
                alv_type: AlvType::Songkhya,
                value: Expr::StringLiteral("হ্যালো".to_string()),
                is_mutable: true,
            }],
        }];

        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&program);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("ধরন")); // "ধরন" in Bangla
    }

    #[test]
    fn test_missing_shuru() {
        let program = vec![Function {
            name: "অন্য_ফাংশন".to_string(),
            params: vec![],
            return_type: AlvType::Void,
            body: vec![],
        }];

        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&program);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("শুরু"));
    }
}
