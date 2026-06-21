use std::path::PathBuf;

use trix_build::{Macros, TrixConfig};

fn main() {
  let vendor_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("vendor/grammars");
  let config = TrixConfig::from_vendor_dir(vendor_dir).unwrap();
  let macros = Macros::from_config(&config).unwrap();
  let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
  std::fs::write(out_dir.join("grammars.rs"), macros.to_string()).unwrap();
}
