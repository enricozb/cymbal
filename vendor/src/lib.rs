use tree_sitter::Language;

unsafe extern "C" {
  fn tree_sitter_vine() -> Language;
}

pub mod vine {
  pub fn language() -> tree_sitter::Language {
    unsafe { crate::tree_sitter_vine() }
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn test_can_load_grammar() {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&super::vine::language()).expect("failed to load vine language");
  }
}
