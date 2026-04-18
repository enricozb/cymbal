use std::{
  env, fs,
  path::{Path, PathBuf},
};

use cymbal::{
  channel,
  config::{Config, Language},
  parser::Parser,
  worker::Worker,
};

fn language_files() -> Vec<String> {
  let mut files: Vec<String> = fs::read_dir(languages_dir())
    .expect("failed to read languages dir")
    .map(|e| {
      e.expect("failed to read dir entry")
        .file_name()
        .into_string()
        .expect("non-UTF-8 filename")
    })
    .collect();

  files.sort();
  files
}

fn languages_dir() -> PathBuf {
  Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/languages")
}

fn snapshots_dir() -> PathBuf {
  Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots")
}

async fn check(filename: &str) {
  let ext = Path::new(filename)
    .extension()
    .and_then(|e| e.to_str())
    .unwrap_or_else(|| panic!("no extension on {filename}"));

  let language = Language::from_extension(ext).unwrap_or_else(|| panic!("no language for extension {ext:?} (file: {filename})"));

  let config: &'static Config = Box::leak(Box::new(Config::default()));

  let path = languages_dir().join(filename);
  let symbol_stream = Parser::new(&path, language, config)
    .symbol_stream()
    .await
    .unwrap_or_else(|e| panic!("failed to create symbol stream for {filename}: {e}"));

  let (_tx, rx) = channel::bounded(1);
  let display_path = Path::new("tests/languages").join(filename);
  let mut worker = Worker::new(None, config, rx, ' ', '\n', Vec::<u8>::new());
  worker.emit_symbols(&display_path, symbol_stream).await.unwrap();
  let snapshot = String::from_utf8(worker.into_writer()).unwrap();

  let snap_path = snapshots_dir().join(format!("{filename}.snap"));

  if env::var("UPDATE_SNAPSHOTS").is_ok() {
    fs::create_dir_all(snapshots_dir()).expect("failed to create snapshots dir");
    fs::write(&snap_path, &snapshot).unwrap_or_else(|e| panic!("failed to write {snap_path:?}: {e}"));
    println!("updated snapshot: {}", snap_path.display());
  } else {
    let expected = fs::read_to_string(&snap_path).unwrap_or_else(|_| {
      panic!(
        "snapshot not found: {}\n\nRun `UPDATE_SNAPSHOTS=1 cargo test` to generate it.",
        snap_path.display()
      )
    });
    assert_eq!(
      snapshot, expected,
      "snapshot mismatch for {filename}\n\nRun `UPDATE_SNAPSHOTS=1 cargo test` to update it.",
    );
  }
}

#[tokio::test]
async fn snapshots() {
  for file in &language_files() {
    check(file).await;
  }
}
