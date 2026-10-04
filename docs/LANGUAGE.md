# আলভীলান Language Reference · ভাষা নির্দেশিকা

This is the complete reference for the language as currently implemented. Every snippet here compiles. The runnable versions are in [`examples/`](../examples).

- [Program structure](#program-structure)
- [Comments](#comments)
- [Types](#types)
- [Variables](#variables)
- [Numbers & digits](#numbers--digits)
- [Operators](#operators)
- [Output](#output)
- [Conditionals](#conditionals)
- [Loops](#loops)
- [Functions](#functions)
- [Lists (arrays)](#lists-arrays)
- [Keyword table](#keyword-table)
- [Errors](#errors)
- [Current limitations](#current-limitations)
- [How the compiler works](#how-the-compiler-works)

## Program structure

Every program needs a function called **`শুরু`** (*start*). It is the entry point, the same as `main` in C.

```
ফাংশন শুরু() {
    দেখাও("হ্যালো, পৃথিবী!")
}
```

Statements are separated by new lines. You don't need semicolons.

## Comments

`//` starts a comment that runs to the end of the line.

```
// এটি একটি মন্তব্য (this is a comment)
```

## Types

| Type | Meaning | Example values |
|---|---|---|
| `সংখ্যা` | whole number (64-bit integer) | `১০`, `-৫`, `0` |
| `দশমিক` | decimal number (64-bit float) | `৩.১৪`, `২.০` |
| `লেখা` | text (string) | `"নমস্কার"` |
| `সত্যমিথ্যা` | true / false | `সত্য`, `মিথ্যা` |
| `তালিকা[ধরন]` | list of one type | `[১, ২, ৩]`, `["ক", "খ"]` |

## Variables

Declare a variable with **`ধরি`** (*let*). The type is always written out:

```
ধরি বয়স: সংখ্যা = ১০
ধরি নাম: লেখা = "রিয়া"
ধরি পাস: সত্যমিথ্যা = সত্য
```

Re-assign without `ধরি`:

```
বয়স = বয়স + ১
```

Names can use Bangla letters, English letters, digits and `_` (e.g. `বয়স_১`, `i`).

## Numbers & digits

You can write numbers with **Bangla digits (০–৯)**, ASCII digits (0–9) or a mix. `১০` and `10` mean the same number.

Printed numbers always use Bangla digits, so the output matches the language:

```
দেখাও(১০ + ২০)   // ৩০
দেখাও(৭.০ / ২.০) // ৩.৫
```

## Operators

| Kind | Operators | Notes |
|---|---|---|
| Arithmetic | `+` `-` `*` `/` `%` | Both sides must have the same type. `/` on `সংখ্যা` truncates (`৭ / ২` → `৩`) |
| Comparison | `==` `!=` `<` `<=` `>` `>=` | Result is `সত্যমিথ্যা` |
| Logic | `এবং` (and), `অথবা` (or), `না` (not) | |
| Unary | `-x`, `না x` | |

Precedence, from lowest to highest: `অথবা` → `এবং` → `== !=` → `< <= > >=` → `+ -` → `* / %` → unary. Use parentheses to group.

## Output

**`দেখাও(…)`** (*show*) prints one value and then a new line. It works with numbers, decimals, text and true/false (printed as `সত্য` / `মিথ্যা`).

## Conditionals

```
যদি (নম্বর >= ৮০) {
    দেখাও("A+")
} নাহলে {
    দেখাও("চেষ্টা চালিয়ে যাও!")
}
```

`নাহলে` (*else*) is optional.

## Loops

**`যতক্ষণ`** (*as long as*) repeats while its condition is true:

```
ধরি ঘর: সংখ্যা = ১
যতক্ষণ (ঘর <= ১০) {
    দেখাও(৫ * ঘর)
    ঘর = ঘর + ১
}
```

## Functions

```
ফাংশন যোগ(ক: সংখ্যা, খ: সংখ্যা) -> সংখ্যা {
    ফেরত ক + খ
}
```

- Parameters are `name: type`, separated by commas.
- `-> type` declares what the function returns. Leave it out if it returns nothing.
- **`ফেরত`** (*return*) gives back a value.
- Recursion works. See [`factorial.alv`](../examples/factorial.alv) and [`fibonacci.alv`](../examples/fibonacci.alv).

## Lists (arrays)

```
ধরি ক: তালিকা[সংখ্যা] = [১০, ২০, ৩০]
দেখাও(ক[০])   // ১০ — indexes start at ০
ক[১] = ১০০    // change an element
```

Every element must have the same type. The compiler checks this.

## Keyword table

| Keyword | Meaning | English equivalent |
|---|---|---|
| `ফাংশন` | define a function | `fn` / `function` |
| `শুরু` | program entry point | `main` |
| `ধরি` | declare a variable | `let` |
| `যদি` / `নাহলে` | if / else | `if` / `else` |
| `যতক্ষণ` | while loop | `while` |
| `ফেরত` | return a value | `return` |
| `দেখাও` | print | `print` |
| `এবং` / `অথবা` / `না` | logical and / or / not | `&&` / `\|\|` / `!` |
| `সত্য` / `মিথ্যা` | true / false | `true` / `false` |
| `নাও` | *reserved* (planned: read input) | `input` |

## Errors

Compiler errors are written in Bangla. They name the variable or token involved and give the line number:

```
ত্রুটি: 'বয়স' চলকটির ধরন হল সংখ্যা, কিন্তু আপনি একে লেখা করার চেষ্টা করছেন।
ত্রুটি [লাইন ২]: ')' প্রত্যাশিত ছিল, কিন্তু '{' পাওয়া গেছে।
```

The compiler checks for:
- undefined variables and functions
- type mismatches in declarations, assignments, returns, arguments and list elements
- arithmetic on mixed types
- a missing `শুরু`

## Current limitations

- Text can't be joined with `+` yet, and numbers can't be converted to text.
- There is no keyboard input yet (`নাও` is reserved for this).
- There are no `for` loops or `break`/`continue`. Use `যতক্ষণ`.
- Lists are fixed-size once created.

## How the compiler works

```
.alv source ──▶ Lexer ──▶ Parser ──▶ Semantic analysis ──▶ LLVM IR ──▶ object file ──▶ cc ──▶ native executable
               (Bangla      (AST)      (types & scopes)     (inkwell)    (host CPU)      ▲
                tokens,                                                                │
                ০–৯ → 0–9)                                       runtime/alv_runtime.c ┘ (Bangla-digit printing)
```

| Stage | File |
|---|---|
| Lexer: Bangla keywords and digits, Unicode identifiers | [`src/frontend/lexer/`](../src/frontend/lexer) |
| Parser: recursive descent with precedence climbing, producing the AST | [`src/frontend/parser/`](../src/frontend/parser) |
| Semantic analysis: scopes, type checking, function signatures | [`src/frontend/semantic/`](../src/frontend/semantic) |
| Code generation: LLVM IR through inkwell, `-O0`…`-O3`, object emission | [`src/backend/codegen/`](../src/backend/codegen) |
| Runtime: prints numbers with Bangla digits | [`runtime/alv_runtime.c`](../runtime/alv_runtime.c) |
| Driver: CLI, linking with `cc` | [`src/main.rs`](../src/main.rs) |

Useful flags:

```bash
alveelan program.alv -o program   # choose the output name
alveelan program.alv -O 2         # optimise (0–3)
alveelan program.alv --emit-ir    # print the generated LLVM IR
```
