//! LLM-generated build.rs which
//! 1. Builds tree-sitter grammars from grammar.js sources, instead of using
//!    pre-built parser.c files.
//! 2. Auto-generates tests which check that the expected symbols exist for
//!    each detected language.

use std::{
  env,
  fmt::Write,
  fs,
  path::{Path, PathBuf},
  process::Command,
};

fn main() {
  let grammars_root = Path::new("grammars");
  let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

  let mut cfg = cc::Build::new();
  let mut bindings = String::new();
  let mut tests = String::new();

  println!("cargo:rerun-if-changed=build.rs");

  writeln!(bindings, "use tree_sitter::Language;\nunsafe extern \"C\" {{").unwrap();

  writeln!(
    tests,
    r"
      #[cfg(test)]
      mod generated_tests {{
        use super::*;
    "
  )
  .unwrap();

  for entry in fs::read_dir(grammars_root).unwrap() {
    let grammar_dir = entry.unwrap().path();
    if !grammar_dir.is_dir() {
      continue;
    }

    let name = grammar_dir.file_name().unwrap().to_string_lossy();
    let fn_name = format!("tree_sitter_{name}");

    let grammar_js = grammar_dir.join("grammar.js");
    println!("cargo:rerun-if-changed={}", grammar_js.display());

    // Always regenerate parser
    let status = Command::new("tree-sitter")
      .arg("generate")
      .current_dir(&grammar_dir)
      .status()
      .expect("failed to run tree-sitter generate");

    assert!(status.success(), "tree-sitter generate failed in {}", grammar_dir.display());

    let parser = grammar_dir.join("src/parser.c");
    if !parser.exists() {
      continue;
    }

    println!("cargo:rerun-if-changed={}", parser.display());

    cfg.include(grammar_dir.join("src")).file(&parser);

    writeln!(bindings, "    fn {fn_name}() -> Language;").unwrap();

    writeln!(
      tests,
      r#"
        #[test]
        fn test_can_load_{name}() {{
          let mut parser = tree_sitter::Parser::new();
          parser
            .set_language(&{name}::language())
            .expect("failed to load {name} language");
        }}
      "#
    )
    .unwrap();
  }

  writeln!(bindings, "}}\n").unwrap();
  writeln!(tests, "}}\n").unwrap();

  // Generate safe Rust wrappers
  for entry in fs::read_dir(grammars_root).unwrap() {
    let grammar_dir = entry.unwrap().path();
    if !grammar_dir.is_dir() {
      continue;
    }

    let name = grammar_dir.file_name().unwrap().to_string_lossy();
    let fn_name = format!("tree_sitter_{name}");

    writeln!(
      bindings,
      r"
        pub mod {name} {{
          use super::{{Language, {fn_name}}};
          #[must_use]
          pub fn language() -> Language {{
            unsafe {{ {fn_name}() }}
          }}
        }}
      "
    )
    .unwrap();
  }

  let bindings_path = out_dir.join("bindings.rs");
  let tests_path = out_dir.join("tests.rs");

  fs::write(bindings_path, bindings).unwrap();
  fs::write(tests_path, tests).unwrap();

  cfg.compile("tree-sitter-multigrammar");
}
