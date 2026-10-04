<div align="center">

<img src="docs/logo.svg" alt="Alveelan logo: অ on the red disc of the Bangladesh flag" width="104">

# আলভীলান · Alveelan

**A Bangla programming language for kids, compiled to fast native code with LLVM.**

Every keyword, type and error message is in Bangla. Write numbers with ০–৯, and your program answers in ০–৯ too.

[![CI](https://github.com/happinessisreal/alveelan/actions/workflows/ci.yml/badge.svg)](https://github.com/happinessisreal/alveelan/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/Rust-2024-000000?logo=rust&logoColor=white)](Cargo.toml)
[![LLVM](https://img.shields.io/badge/LLVM-18--21-262D3A?logo=llvm&logoColor=white)](#-install)
[![License: MIT](https://img.shields.io/badge/license-MIT-yellow.svg)](LICENSE)

**English** · [বাংলা](README.bn.md)

<img src="docs/demo.png" alt="Terminal: a Fibonacci program written in Bangla is compiled with alveelan and prints the first ten Fibonacci numbers in Bangla digits" width="820">

</div>

## Why

For many young learners in Bangladesh, English keywords are an extra hurdle on top of learning to program. Alveelan removes it:

- **The whole language is in Bangla.** `ধরি` (let), `যদি`/`নাহলে` (if/else), `যতক্ষণ` (while), `ফাংশন` (function), `দেখাও` (print). Types are `সংখ্যা` (whole number), `দশমিক` (decimal), `লেখা` (text), `সত্যমিথ্যা` (true/false) and `তালিকা[…]` (list).
- **Numbers in Bangla digits.** Programs accept `১০` or `10`, and printed results come back as ৩০, ১২০ and ৩.৫.
- **Friendly Bangla error messages.** They give the line number and name the variable or type involved, so learners can fix mistakes on their own.
- **A real compiler.** It does more than interpret: the source goes through a lexer, parser, type checker and LLVM, and comes out as a native executable.

<p align="center"><img src="docs/errors.png" alt="Bangla compiler error messages for a type mismatch, an undefined variable and a syntax error with its line number" width="760"></p>

## 🚀 Install

Prerequisites:
- [Rust](https://rustup.rs) (1.85+)
- a C compiler (`cc`)
- **LLVM 18, 19, 20 or 21** with development headers

```bash
# Ubuntu / Debian
sudo apt install llvm-18-dev libpolly-18-dev libzstd-dev clang

git clone https://github.com/happinessisreal/alveelan.git
cd alveelan
./install.sh          # detects your LLVM version, builds, installs to ~/.local/bin
```

<details>
<summary>Manual build / other LLVM versions</summary>

```bash
cargo build --release                                            # LLVM 18 (default)
cargo build --release --no-default-features --features llvm20    # or llvm19 / llvm21
sudo cp target/release/alveelan /usr/local/bin/
```

If the build can't find LLVM, point `llvm-sys` at it, e.g. `export LLVM_SYS_181_PREFIX=/usr/lib/llvm-18`.

</details>

## ✏️ Your first program

```
// hello.alv
ফাংশন শুরু() {
    দেখাও("হ্যালো, পৃথিবী!")
}
```

```bash
alveelan hello.alv     # অভিনন্দন! 'hello' তৈরি করা হয়েছে।
./hello                # হ্যালো, পৃথিবী!
```

## 🧩 A quick tour

```
ফাংশন ফ্যাক্টোরিয়াল(এন: সংখ্যা) -> সংখ্যা {       // a function that returns a number
    যদি (এন <= ১) {
        ফেরত ১
    } নাহলে {
        ফেরত এন * ফ্যাক্টোরিয়াল(এন - ১)        // recursion works
    }
}

ফাংশন শুরু() {
    ধরি ক: তালিকা[সংখ্যা] = [১০, ২০, ৩০]        // a list of numbers
    ক[১] = ১০০
    দেখাও(ক[১])                                  // ১০০

    ধরি ঘর: সংখ্যা = ১
    যতক্ষণ (ঘর <= ৩) {                            // while loop
        যদি (না (ঘর == ২)) {                        // skip ২
            দেখাও(ফ্যাক্টোরিয়াল(ঘর + ৪))         // ১২০, then ৫০৪০
        }
        ঘর = ঘর + ১
    }
}
```

The full syntax, operators, keyword table and current limitations are in **[docs/LANGUAGE.md](docs/LANGUAGE.md)**.

## 📚 Examples

| File | Shows |
|---|---|
| [`hello.alv`](examples/hello.alv) | printing text |
| [`math.alv`](examples/math.alv) | variables, arithmetic, `যদি` / `নাহলে` |
| [`array.alv`](examples/array.alv) | lists of numbers and text, indexing, updating |
| [`factorial.alv`](examples/factorial.alv) | functions and recursion |
| [`fibonacci.alv`](examples/fibonacci.alv) | recursion inside a `যতক্ষণ` loop |
| [`multiplication_table.alv`](examples/multiplication_table.alv) | the ৫ times table (নামতা) with a loop |
| [`even_odd.alv`](examples/even_odd.alv) | `%`, `এবং`, `অথবা`, `না`, boolean functions |
| [`circle.alv`](examples/circle.alv) | `দশমিক` (decimal) arithmetic |

Every example is compiled and run in CI, and its output is checked against [`tests/expected/`](tests/expected).

## ⚙️ How it works

```
.alv ─▶ Lexer ─▶ Parser ─▶ Semantic analysis ─▶ LLVM IR ─▶ object file ─▶ cc + runtime ─▶ executable
```

| Stage | Where |
|---|---|
| **Lexer.** Bangla keywords, Unicode identifiers, ০–৯ digits normalised to 0–9 | [`src/frontend/lexer`](src/frontend/lexer) |
| **Parser.** Recursive descent with precedence climbing, producing an AST | [`src/frontend/parser`](src/frontend/parser) |
| **Semantic analysis.** Scopes, type checking, function signatures, Bangla diagnostics | [`src/frontend/semantic`](src/frontend/semantic) |
| **Code generation.** LLVM IR through [inkwell](https://github.com/TheDan64/inkwell), optimisation levels `-O0`…`-O3` | [`src/backend/codegen`](src/backend/codegen) |
| **Runtime.** A tiny C library that prints numbers with Bangla digits | [`runtime/alv_runtime.c`](runtime/alv_runtime.c) |

```bash
alveelan program.alv -o program   # output name
alveelan program.alv -O 2         # optimise
alveelan program.alv --emit-ir    # inspect the generated LLVM IR
```

## 🧪 Development

```bash
cargo test --release    # 14 unit tests + end-to-end tests that compile and run every example
cargo fmt && cargo clippy
```

## 🗺️ Roadmap

- [ ] `নাও`: read input from the keyboard
- [ ] Joining text with `+`, and converting numbers to text
- [ ] `for`-style loops, `থামো` (break)
- [ ] A browser playground for classrooms

## 📄 License

[MIT](LICENSE)

শুভ প্রোগ্রামিং! 🎈
