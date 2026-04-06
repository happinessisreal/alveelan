use clap::Parser as ClapParser;
use inkwell::OptimizationLevel;
use inkwell::context::Context;
use std::fs;
use std::path::Path;
use std::process::Command;

mod backend;
mod frontend;

use crate::backend::codegen::CodeGenerator;
use crate::frontend::lexer::Lexer;
use crate::frontend::parser::Parser;
use crate::frontend::semantic::SemanticAnalyzer;

#[derive(ClapParser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// ইনপুট .alv ফাইল (Input .alv file)
    input: String,

    /// আউটপুট ফাইলের নাম (Output file name)
    #[arg(short, long)]
    output: Option<String>,

    /// LLVM IR আউটপুট দেখান (Emit LLVM IR)
    #[arg(long)]
    emit_ir: bool,

    /// অপ্টিমাইজেশন লেভেল (Optimization level: 0, 1, 2, 3)
    #[arg(short = 'O', default_value = "0")]
    opt_level: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let input_path = Path::new(&args.input);
    if input_path.extension().and_then(|s| s.to_str()) != Some("alv") {
        return Err("ইনপুট ফাইলটি অবশ্যই .alv ফরম্যাটের হতে হবে। (Input must be .alv)".into());
    }

    let source = fs::read_to_string(&args.input)?;

    // 1. Lex
    let lexer = Lexer::new(&source);

    // 2. Parse
    let mut parser = Parser::new(lexer);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    // 3. Semantic Analysis
    let mut analyzer = SemanticAnalyzer::new();
    if let Err(e) = analyzer.analyze(&program) {
        eprintln!("{}", e);
        std::process::exit(1);
    }

    // 4. Code Generation
    let context = Context::create();
    let module = context.create_module("alveelan");
    let builder = context.create_builder();

    let opt_level = match args.opt_level {
        0 => OptimizationLevel::None,
        1 => OptimizationLevel::Less,
        2 => OptimizationLevel::Default,
        3 => OptimizationLevel::Aggressive,
        _ => OptimizationLevel::None,
    };

    let mut codegen = CodeGenerator::new(&context, &module, &builder, opt_level);

    if let Err(e) = codegen.compile(&program) {
        eprintln!("{}", e);
        std::process::exit(1);
    }

    if args.emit_ir {
        module.print_to_stderr();
    }

    // 5. Output Object File
    let output_name = args.output.unwrap_or_else(|| {
        input_path
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    });

    let obj_file = format!("{}.o", output_name);
    codegen.emit_object_file(&obj_file)?;

    // 6. Link
    let status = Command::new("cc")
        .arg(&obj_file)
        .arg("-o")
        .arg(&output_name)
        .arg("-no-pie") // Often needed for simple LLVM-emitted objects
        .status()?;

    if !status.success() {
        return Err("লিঙ্কিং করা সম্ভব হয়নি। (Linking failed)".into());
    }

    // Cleanup object file
    fs::remove_file(obj_file)?;

    println!(
        "অভিনন্দন! '{}' তৈরি করা হয়েছে। (Successfully created '{}')",
        output_name, output_name
    );

    Ok(())
}
