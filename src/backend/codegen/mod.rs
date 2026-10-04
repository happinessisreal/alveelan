use crate::frontend::parser::ast::*;
use inkwell::OptimizationLevel;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum};
use inkwell::values::{
    BasicMetadataValueEnum, BasicValue, BasicValueEnum, FunctionValue, PointerValue, ValueKind,
};
use std::collections::HashMap;

pub struct CodeGenerator<'a, 'ctx> {
    pub context: &'ctx Context,
    pub module: &'a Module<'ctx>,
    pub builder: &'a Builder<'ctx>,
    variables: HashMap<String, (PointerValue<'ctx>, AlvType)>,
    opt_level: OptimizationLevel,
}

impl<'a, 'ctx> CodeGenerator<'a, 'ctx> {
    pub fn new(
        context: &'ctx Context,
        module: &'a Module<'ctx>,
        builder: &'a Builder<'ctx>,
        opt_level: OptimizationLevel,
    ) -> Self {
        Self {
            context,
            module,
            builder,
            variables: HashMap::new(),
            opt_level,
        }
    }

    pub fn compile(&mut self, program: &[Function]) -> Result<(), String> {
        // First pass: declare all functions
        for func in program {
            self.declare_function(func)?;
        }

        // Second pass: compile bodies
        for func in program {
            self.compile_function(func)?;
        }

        Ok(())
    }

    fn declare_function(&mut self, func: &Function) -> Result<FunctionValue<'ctx>, String> {
        let ret_type = self.alv_to_llvm_type(&func.return_type);
        let arg_types: Vec<BasicMetadataTypeEnum> = func
            .params
            .iter()
            .map(|p| self.alv_to_llvm_type(&p.alv_type).into())
            .collect();

        // Rename শুরু to main for the system linker and ensure it returns i32
        let is_main = func.name == "শুরু";
        let llvm_name = if is_main { "main" } else { &func.name };

        let fn_type = if is_main {
            self.context.i32_type().fn_type(&arg_types, false)
        } else if func.return_type == AlvType::Void {
            self.context.void_type().fn_type(&arg_types, false)
        } else {
            match ret_type {
                BasicTypeEnum::IntType(t) => t.fn_type(&arg_types, false),
                BasicTypeEnum::FloatType(t) => t.fn_type(&arg_types, false),
                BasicTypeEnum::PointerType(t) => t.fn_type(&arg_types, false),
                _ => return Err("Unsupported return type".to_string()),
            }
        };

        let fn_val = self.module.add_function(llvm_name, fn_type, None);
        Ok(fn_val)
    }

    fn compile_function(&mut self, func: &Function) -> Result<(), String> {
        let llvm_name = if func.name == "শুরু" {
            "main"
        } else {
            &func.name
        };
        let fn_val = self.module.get_function(llvm_name).unwrap();
        let entry = self.context.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(entry);

        self.variables.clear();
        for (i, arg) in fn_val.get_param_iter().enumerate() {
            let param = &func.params[i];
            let alloca = self.create_entry_block_alloca(fn_val, &param.name, &param.alv_type);
            self.builder
                .build_store(alloca, arg)
                .map_err(|e| e.to_string())?;
            self.variables
                .insert(param.name.clone(), (alloca, param.alv_type.clone()));
        }

        for stmt in &func.body {
            self.compile_statement(stmt, fn_val)?;
        }

        if func.name == "শুরু" {
            let current_bb = self.builder.get_insert_block().unwrap();
            if current_bb.get_terminator().is_none() {
                let zero = self.context.i32_type().const_int(0, false);
                self.builder
                    .build_return(Some(&zero))
                    .map_err(|e| e.to_string())?;
            }
        } else if func.return_type == AlvType::Void {
            let current_bb = self.builder.get_insert_block().unwrap();
            if current_bb.get_terminator().is_none() {
                self.builder.build_return(None).map_err(|e| e.to_string())?;
            }
        } else {
            let current_bb = self.builder.get_insert_block().unwrap();
            if current_bb.get_terminator().is_none() {
                let default_val: BasicValueEnum = match func.return_type {
                    AlvType::Songkhya => self.context.i64_type().const_int(0, false).into(),
                    AlvType::Doshomik => self.context.f64_type().const_float(0.0).into(),
                    AlvType::SottoMittha => self.context.bool_type().const_int(0, false).into(),
                    _ => self.context.i64_type().const_int(0, false).into(),
                };
                let ret_val: &dyn BasicValue = &default_val;
                self.builder
                    .build_return(Some(ret_val))
                    .map_err(|e| e.to_string())?;
            }
        }

        if !fn_val.verify(true) {
            return Err(format!(
                "ত্রুটি: '{}' ফাংশনটি সঠিকভাবে তৈরি করা যায়নি।",
                func.name
            ));
        }

        Ok(())
    }

    fn compile_statement(
        &mut self,
        stmt: &Stmt,
        fn_val: FunctionValue<'ctx>,
    ) -> Result<(), String> {
        match stmt {
            Stmt::Let {
                name,
                alv_type,
                value,
                ..
            } => {
                let val = self.compile_expression(value)?;
                let alloca = self.create_entry_block_alloca(fn_val, name, alv_type);
                self.builder
                    .build_store(alloca, val)
                    .map_err(|e| e.to_string())?;
                self.variables
                    .insert(name.clone(), (alloca, alv_type.clone()));
            }
            Stmt::Assignment { name, value } => {
                let val = self.compile_expression(value)?;
                let (alloca, _) = self
                    .variables
                    .get(name)
                    .ok_or(format!("ত্রুটি: '{}' চলকটি পাওয়া যায়নি।", name))?;
                self.builder
                    .build_store(*alloca, val)
                    .map_err(|e| e.to_string())?;
            }
            Stmt::Print(expr) => {
                let val = self.compile_expression(expr)?;
                self.call_print(val)?;
            }
            Stmt::Return(expr) => {
                let val = if let Some(e) = expr {
                    Some(self.compile_expression(e)?)
                } else {
                    None
                };
                let ret_val: Option<&dyn BasicValue> = match val {
                    Some(ref v) => Some(v as &dyn BasicValue),
                    None => None,
                };
                self.builder
                    .build_return(ret_val)
                    .map_err(|e| e.to_string())?;
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond = self.compile_expression(condition)?.into_int_value();

                let then_bb = self.context.append_basic_block(fn_val, "then");
                let else_bb = self.context.append_basic_block(fn_val, "else");
                let merge_bb = self.context.append_basic_block(fn_val, "ifcont");

                self.builder
                    .build_conditional_branch(cond, then_bb, else_bb)
                    .map_err(|e| e.to_string())?;

                // Then
                self.builder.position_at_end(then_bb);
                for s in then_branch {
                    self.compile_statement(s, fn_val)?;
                }
                // Nested control flow may have moved the builder to another block,
                // so check the *current* block, not the one we started in.
                if self.current_block_open() {
                    self.builder
                        .build_unconditional_branch(merge_bb)
                        .map_err(|e| e.to_string())?;
                }

                // Else
                self.builder.position_at_end(else_bb);
                if let Some(branch) = else_branch {
                    for s in branch {
                        self.compile_statement(s, fn_val)?;
                    }
                }
                if self.current_block_open() {
                    self.builder
                        .build_unconditional_branch(merge_bb)
                        .map_err(|e| e.to_string())?;
                }

                self.builder.position_at_end(merge_bb);
            }
            Stmt::While { condition, body } => {
                let cond_bb = self.context.append_basic_block(fn_val, "whilecond");
                let body_bb = self.context.append_basic_block(fn_val, "whilebody");
                let after_bb = self.context.append_basic_block(fn_val, "afterwhile");

                self.builder
                    .build_unconditional_branch(cond_bb)
                    .map_err(|e| e.to_string())?;
                self.builder.position_at_end(cond_bb);

                let cond = self.compile_expression(condition)?.into_int_value();
                self.builder
                    .build_conditional_branch(cond, body_bb, after_bb)
                    .map_err(|e| e.to_string())?;

                self.builder.position_at_end(body_bb);
                for s in body {
                    self.compile_statement(s, fn_val)?;
                }
                if self.current_block_open() {
                    self.builder
                        .build_unconditional_branch(cond_bb)
                        .map_err(|e| e.to_string())?;
                }

                self.builder.position_at_end(after_bb);
            }
            Stmt::Expression(expr) => {
                self.compile_expression(expr)?;
            }
            Stmt::IndexAssignment { target, value } => {
                // target must be an IndexAccess
                if let Expr::IndexAccess(array_expr, index_expr) = &**target {
                    let array_ptr = self.compile_expression(array_expr)?.into_pointer_value();
                    let index = self.compile_expression(index_expr)?.into_int_value();
                    let val = self.compile_expression(value)?;

                    let element_type = match &**array_expr {
                        Expr::Variable(name) => {
                            let (_, alv_type) = self
                                .variables
                                .get(name)
                                .ok_or(format!("ত্রুটি: '{}' চলকটি পাওয়া যায়নি।", name))?;
                            match alv_type {
                                AlvType::Array(inner) => self.alv_to_llvm_type(inner),
                                _ => {
                                    return Err(
                                        "ব্যর্থ: শুধুমাত্র তালিকাতে ইনডেক্স ব্যবহার করা সম্ভব।".to_string()
                                    );
                                }
                            }
                        }
                        _ => val.get_type(), // Fallback for nested or literals
                    };

                    let zero = self.context.i64_type().const_int(0, false);
                    let element_ptr = unsafe {
                        self.builder.build_gep(
                            element_type.array_type(0),
                            array_ptr,
                            &[zero, index],
                            "ptr",
                        )
                    }
                    .map_err(|e| e.to_string())?;

                    self.builder
                        .build_store(element_ptr, val)
                        .map_err(|e| e.to_string())?;
                }
            }
        }
        Ok(())
    }

    fn compile_expression(&self, expr: &Expr) -> Result<BasicValueEnum<'ctx>, String> {
        match expr {
            Expr::IntLiteral(n) => Ok(self.context.i64_type().const_int(*n as u64, false).into()),
            Expr::FloatLiteral(n) => Ok(self.context.f64_type().const_float(*n).into()),
            Expr::StringLiteral(s) => {
                let global = self
                    .builder
                    .build_global_string_ptr(s, "str")
                    .map_err(|e| e.to_string())?;
                Ok(global.as_basic_value_enum())
            }
            Expr::BoolLiteral(b) => Ok(self.context.bool_type().const_int(*b as u64, false).into()),
            Expr::Variable(name) => {
                let (alloca, alv_type) = self
                    .variables
                    .get(name)
                    .ok_or(format!("ত্রুটি: '{}' চলকটি পাওয়া যায়নি।", name))?;
                let llvm_type = self.alv_to_llvm_type(alv_type);
                self.builder
                    .build_load(llvm_type, *alloca, name)
                    .map_err(|e| e.to_string())
            }
            Expr::Binary(op, left, right) => {
                let lhs = self.compile_expression(left)?;
                let rhs = self.compile_expression(right)?;

                match (lhs, rhs) {
                    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => match op {
                        BinaryOp::Add => Ok(self
                            .builder
                            .build_int_add(l, r, "addtmp")
                            .map_err(|e| e.to_string())?
                            .into()),
                        BinaryOp::Sub => Ok(self
                            .builder
                            .build_int_sub(l, r, "subtmp")
                            .map_err(|e| e.to_string())?
                            .into()),
                        BinaryOp::Mul => Ok(self
                            .builder
                            .build_int_mul(l, r, "multmp")
                            .map_err(|e| e.to_string())?
                            .into()),
                        BinaryOp::Div => Ok(self
                            .builder
                            .build_int_signed_div(l, r, "divtmp")
                            .map_err(|e| e.to_string())?
                            .into()),
                        BinaryOp::Mod => Ok(self
                            .builder
                            .build_int_signed_rem(l, r, "modtmp")
                            .map_err(|e| e.to_string())?
                            .into()),
                        BinaryOp::Eq => Ok(self
                            .builder
                            .build_int_compare(inkwell::IntPredicate::EQ, l, r, "eqtmp")
                            .map_err(|e| e.to_string())?
                            .into()),
                        BinaryOp::Ne => Ok(self
                            .builder
                            .build_int_compare(inkwell::IntPredicate::NE, l, r, "netmp")
                            .map_err(|e| e.to_string())?
                            .into()),
                        BinaryOp::Lt => Ok(self
                            .builder
                            .build_int_compare(inkwell::IntPredicate::SLT, l, r, "lttmp")
                            .map_err(|e| e.to_string())?
                            .into()),
                        BinaryOp::Le => Ok(self
                            .builder
                            .build_int_compare(inkwell::IntPredicate::SLE, l, r, "letmp")
                            .map_err(|e| e.to_string())?
                            .into()),
                        BinaryOp::Gt => Ok(self
                            .builder
                            .build_int_compare(inkwell::IntPredicate::SGT, l, r, "gttmp")
                            .map_err(|e| e.to_string())?
                            .into()),
                        BinaryOp::Ge => Ok(self
                            .builder
                            .build_int_compare(inkwell::IntPredicate::SGE, l, r, "getmp")
                            .map_err(|e| e.to_string())?
                            .into()),
                        BinaryOp::And => Ok(self
                            .builder
                            .build_and(l, r, "andtmp")
                            .map_err(|e| e.to_string())?
                            .into()),
                        BinaryOp::Or => Ok(self
                            .builder
                            .build_or(l, r, "ortmp")
                            .map_err(|e| e.to_string())?
                            .into()),
                    },
                    (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => match op {
                        BinaryOp::Add => Ok(self
                            .builder
                            .build_float_add(l, r, "addtmp")
                            .map_err(|e| e.to_string())?
                            .into()),
                        BinaryOp::Sub => Ok(self
                            .builder
                            .build_float_sub(l, r, "subtmp")
                            .map_err(|e| e.to_string())?
                            .into()),
                        BinaryOp::Mul => Ok(self
                            .builder
                            .build_float_mul(l, r, "multmp")
                            .map_err(|e| e.to_string())?
                            .into()),
                        BinaryOp::Div => Ok(self
                            .builder
                            .build_float_div(l, r, "divtmp")
                            .map_err(|e| e.to_string())?
                            .into()),
                        BinaryOp::Eq => Ok(self
                            .builder
                            .build_float_compare(inkwell::FloatPredicate::OEQ, l, r, "eqtmp")
                            .map_err(|e| e.to_string())?
                            .into()),
                        _ => Err("ত্রুটি: দশমিক সংখ্যার সাথে এই কাজটি করা সম্ভব নয়।".to_string()),
                    },
                    _ => Err("ত্রুটি: অমিল ধরণের মধ্যে এই কাজটি করা সম্ভব নয়।".to_string()),
                }
            }
            Expr::Unary(op, expr) => {
                let val = self.compile_expression(expr)?;
                match op {
                    UnaryOp::Neg => match val {
                        BasicValueEnum::IntValue(v) => Ok(self
                            .builder
                            .build_int_neg(v, "negtmp")
                            .map_err(|e| e.to_string())?
                            .into()),
                        BasicValueEnum::FloatValue(v) => Ok(self
                            .builder
                            .build_float_neg(v, "negtmp")
                            .map_err(|e| e.to_string())?
                            .into()),
                        _ => Err("শৈল্পিক ত্রুটি".to_string()),
                    },
                    UnaryOp::Not => match val {
                        BasicValueEnum::IntValue(v) => Ok(self
                            .builder
                            .build_not(v, "nottmp")
                            .map_err(|e| e.to_string())?
                            .into()),
                        _ => Err("শৈল্পিক ত্রুটি".to_string()),
                    },
                }
            }
            Expr::Call(name, args) => {
                let func = self
                    .module
                    .get_function(name)
                    .ok_or(format!("ত্রুটি: '{}' ফাংশনটি পাওয়া যায়নি।", name))?;
                let compiled_args: Vec<BasicMetadataValueEnum> = args
                    .iter()
                    .map(|arg| Ok(self.compile_expression(arg)?.into()))
                    .collect::<Result<Vec<_>, String>>()?;

                let call = self
                    .builder
                    .build_call(func, &compiled_args, "calltmp")
                    .map_err(|e| e.to_string())?;

                match call.try_as_basic_value() {
                    ValueKind::Basic(v) => Ok(v),
                    ValueKind::Instruction(_) => {
                        Ok(self.context.i64_type().const_int(0, false).into())
                    }
                }
            }
            Expr::ArrayLiteral(elements) => {
                if elements.is_empty() {
                    return Err("Empty array not supported in codegen".to_string());
                }

                let compiled_elements: Vec<BasicValueEnum> = elements
                    .iter()
                    .map(|e| self.compile_expression(e))
                    .collect::<Result<Vec<_>, String>>()?;

                let element_type = compiled_elements[0].get_type();
                let array_type = element_type.array_type(compiled_elements.len() as u32);

                // Allocate on stack
                let array_ptr = self
                    .builder
                    .build_alloca(array_type, "array_lit")
                    .map_err(|e| e.to_string())?;

                let zero = self.context.i64_type().const_int(0, false);
                for (i, val) in compiled_elements.iter().enumerate() {
                    let index = self.context.i64_type().const_int(i as u64, false);
                    let element_ptr = unsafe {
                        self.builder
                            .build_gep(array_type, array_ptr, &[zero, index], "elem")
                            .map_err(|e| e.to_string())?
                    };
                    self.builder
                        .build_store(element_ptr, *val)
                        .map_err(|e| e.to_string())?;
                }

                Ok(array_ptr.as_basic_value_enum())
            }
            Expr::IndexAccess(array_expr, index_expr) => {
                let array_val = self.compile_expression(array_expr)?;
                let array_ptr = array_val.into_pointer_value();
                let index = self.compile_expression(index_expr)?.into_int_value();

                let element_type = match &**array_expr {
                    Expr::Variable(name) => {
                        let (_, alv_type) = self
                            .variables
                            .get(name)
                            .ok_or(format!("ত্রুটি: '{}' চলকটি পাওয়া যায়নি।", name))?;
                        match alv_type {
                            AlvType::Array(inner) => self.alv_to_llvm_type(inner),
                            _ => return Err("ব্যর্থ: শুধুমাত্র তালিকাতে ইনডেক্স ব্যবহার করা সম্ভব।".to_string()),
                        }
                    }
                    _ => self.context.i64_type().into(), // Fallback
                };

                let zero = self.context.i64_type().const_int(0, false);
                let element_ptr = unsafe {
                    self.builder
                        .build_gep(element_type.array_type(0), array_ptr, &[zero, index], "ptr")
                        .map_err(|e| e.to_string())?
                };

                self.builder
                    .build_load(element_type, element_ptr, "val")
                    .map_err(|e| e.to_string())
            }
        }
    }

    fn alv_to_llvm_type(&self, alv_type: &AlvType) -> BasicTypeEnum<'ctx> {
        match alv_type {
            AlvType::Songkhya => self.context.i64_type().into(),
            AlvType::Doshomik => self.context.f64_type().into(),
            AlvType::Lekha => self.context.ptr_type(inkwell::AddressSpace::from(0)).into(),
            AlvType::SottoMittha => self.context.bool_type().into(),
            AlvType::Array(_) => self.context.ptr_type(inkwell::AddressSpace::from(0)).into(),
            AlvType::Void => self.context.i8_type().into(), // placeholder
        }
    }

    fn create_entry_block_alloca(
        &self,
        func: FunctionValue<'ctx>,
        name: &str,
        alv_type: &AlvType,
    ) -> PointerValue<'ctx> {
        let builder = self.context.create_builder();
        let entry = func.get_first_basic_block().unwrap();
        match entry.get_first_instruction() {
            Some(instr) => builder.position_before(&instr),
            None => builder.position_at_end(entry),
        }
        let ty = self.alv_to_llvm_type(alv_type);
        builder.build_alloca(ty, name).unwrap()
    }

    fn call_print(&self, val: BasicValueEnum<'ctx>) -> Result<(), String> {
        let printf = self.module.get_function("printf").unwrap_or_else(|| {
            let i32_type = self.context.i32_type();
            let char_ptr = self.context.ptr_type(inkwell::AddressSpace::from(0));
            let printf_type = i32_type.fn_type(&[char_ptr.into()], true);
            self.module.add_function("printf", printf_type, None)
        });

        match val {
            BasicValueEnum::IntValue(v) => {
                if v.get_type().get_bit_width() == 1 {
                    // Boolean
                    let true_str = self
                        .builder
                        .build_global_string_ptr("সত্য\n", "s_true")
                        .unwrap();
                    let false_str = self
                        .builder
                        .build_global_string_ptr("মিথ্যা\n", "s_false")
                        .unwrap();
                    let str_ptr = self
                        .builder
                        .build_select(
                            v,
                            true_str.as_basic_value_enum().into_pointer_value(),
                            false_str.as_basic_value_enum().into_pointer_value(),
                            "sel",
                        )
                        .unwrap();
                    self.builder
                        .build_call(printf, &[str_ptr.into()], "call")
                        .unwrap();
                } else {
                    // Printed in Bangla digits by the runtime (see runtime/alv_runtime.c).
                    let i64_type = self.context.i64_type();
                    let print_int = self.runtime_fn("alv_print_int", i64_type.into());
                    let v = if v.get_type().get_bit_width() == 64 {
                        v
                    } else {
                        self.builder.build_int_s_extend(v, i64_type, "ext").unwrap()
                    };
                    self.builder
                        .build_call(print_int, &[v.into()], "call")
                        .unwrap();
                }
            }
            BasicValueEnum::FloatValue(v) => {
                let f64_type = self.context.f64_type();
                let print_float = self.runtime_fn("alv_print_float", f64_type.into());
                self.builder
                    .build_call(print_float, &[v.into()], "call")
                    .unwrap();
            }
            BasicValueEnum::PointerValue(v) => {
                let format = self
                    .builder
                    .build_global_string_ptr("%s\n", "f_str")
                    .unwrap();
                self.builder
                    .build_call(
                        printf,
                        &[format.as_basic_value_enum().into(), v.into()],
                        "call",
                    )
                    .unwrap();
            }
            _ => return Err("ত্রুটি: এই ধরণের মান দেখানো সম্ভব নয়।".to_string()),
        }
        Ok(())
    }

    /// True if the builder's current block still needs a terminator.
    fn current_block_open(&self) -> bool {
        self.builder
            .get_insert_block()
            .is_some_and(|bb| bb.get_terminator().is_none())
    }

    /// Declare (once) a `void name(arg)` function provided by the C runtime.
    fn runtime_fn(&self, name: &str, arg: BasicMetadataTypeEnum<'ctx>) -> FunctionValue<'ctx> {
        self.module.get_function(name).unwrap_or_else(|| {
            let fn_type = self.context.void_type().fn_type(&[arg], false);
            self.module.add_function(name, fn_type, None)
        })
    }

    pub fn emit_object_file(&self, path: &str) -> Result<(), String> {
        Target::initialize_all(&InitializationConfig::default());
        let triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple).map_err(|e| e.to_string())?;

        let cpu = TargetMachine::get_host_cpu_name().to_string();
        let features = TargetMachine::get_host_cpu_features().to_string();

        let tm = target
            .create_target_machine(
                &triple,
                &cpu,
                &features,
                self.opt_level,
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or("Failed to create target machine".to_string())?;

        tm.write_to_file(self.module, FileType::Object, path.as_ref())
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
