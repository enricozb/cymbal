- shard sqlite file into ~8 files
  - modulo modified time as a hash
  - this should hopefully making writing to cache from a fresh large repo faster

- add default config inheritance
  ```toml
  [default]
  languages = true
  # or
  languages = ["rust", "python", "c"]
  ```

- vendor tree-sitter grammars that aren't published to crates.io
  - toml
  - fish
