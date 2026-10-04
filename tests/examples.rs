//! End-to-end tests: compile every `examples/*.alv` with the real compiler, run the
//! produced executable and compare its stdout with `tests/expected/<name>.txt`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn compiler() -> &'static str {
    env!("CARGO_BIN_EXE_alveelan")
}

fn scratch_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("alveelan-test-{}-{}", name, std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Compile `src`, run the binary, return its stdout.
fn compile_and_run(src: &Path, name: &str) -> String {
    let dir = scratch_dir(name);
    let exe = dir.join(name);
    let out = Command::new(compiler())
        .arg(src)
        .arg("-o")
        .arg(&exe)
        .output()
        .expect("failed to launch compiler");
    assert!(
        out.status.success(),
        "compiling {} failed:\n{}",
        src.display(),
        String::from_utf8_lossy(&out.stderr)
    );
    let run = Command::new(&exe)
        .output()
        .expect("failed to run compiled program");
    assert!(run.status.success(), "{} exited with {}", name, run.status);
    let _ = fs::remove_dir_all(&dir);
    String::from_utf8(run.stdout).expect("program output is not UTF-8")
}

#[test]
fn all_examples_produce_expected_output() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut checked = 0;
    for entry in fs::read_dir(root.join("examples")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("alv") {
            continue;
        }
        let name = path.file_stem().unwrap().to_str().unwrap().to_string();
        let expected_path = root.join("tests/expected").join(format!("{name}.txt"));
        let expected = fs::read_to_string(&expected_path)
            .unwrap_or_else(|_| panic!("missing {}", expected_path.display()));
        assert_eq!(
            compile_and_run(&path, &name),
            expected,
            "output mismatch for {name}.alv"
        );
        checked += 1;
    }
    assert!(
        checked >= 8,
        "expected at least 8 examples, found {checked}"
    );
}

/// Compile a snippet that must fail; return the compiler's stderr.
fn compile_error(name: &str, source: &str) -> String {
    let dir = scratch_dir(name);
    let src = dir.join(format!("{name}.alv"));
    fs::write(&src, source).unwrap();
    let out = Command::new(compiler())
        .arg(&src)
        .arg("-o")
        .arg(dir.join(name))
        .output()
        .unwrap();
    let _ = fs::remove_dir_all(&dir);
    assert!(!out.status.success(), "{name} should not compile");
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn undefined_variable_is_reported() {
    let err = compile_error("undef", "ফাংশন শুরু() {\n    দেখাও(অজানা)\n}\n");
    assert!(err.contains("'অজানা'"), "{err}");
}

#[test]
fn type_mismatch_uses_bangla_type_names() {
    let err = compile_error("types", "ফাংশন শুরু() {\n    ধরি ক: সংখ্যা = \"লেখা\"\n}\n");
    assert!(err.contains("সংখ্যা") && err.contains("লেখা"), "{err}");
}

#[test]
fn syntax_error_reports_line_in_bangla_digits() {
    let err = compile_error("syntax", "ফাংশন শুরু() {\n    দেখাও(\"হ্যালো\"\n}\n");
    assert!(err.contains("লাইন ৩"), "{err}");
}

#[test]
fn rejects_non_alv_input() {
    let out = Command::new(compiler())
        .arg("program.txt")
        .output()
        .unwrap();
    assert!(!out.status.success());
}
