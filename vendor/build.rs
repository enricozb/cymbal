use std::{fs, path::Path};

fn main() {
  let grammars_root = Path::new("grammars");
  let mut cfg = cc::Build::new();

  for entry in fs::read_dir(grammars_root).unwrap() {
    let grammar_dir = entry.unwrap().path();
    let src_dir = grammar_dir.join("src");

    let parser = src_dir.join("parser.c");
    if !parser.exists() {
      continue;
    }

    println!("cargo:rerun-if-changed={}", parser.display());

    cfg.include(&src_dir).file(&parser);

    let scanner = src_dir.join("scanner.c");
    if scanner.exists() {
      println!("cargo:rerun-if-changed={}", scanner.display());
      cfg.file(scanner);
    }
  }

  cfg.compile("tree-sitter-multigrammar");
}
