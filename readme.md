# cymbal - list symbols in a codebase

## Overview
`cymbal` lists all symbols in a codebase. For example (in bash),
```
cymbal --language rust
rs   type    ./cymbal/src/utils.rs 7 9  Lazy
rs   struct  ./cymbal/src/utils.rs 11 11  RawPath
rs   method  ./cymbal/src/utils.rs 14 5 RawPath:: from
rs   method  ./cymbal/src/utils.rs 20 5 PathBuf:: from
rs   impl    ./cymbal/src/utils.rs 13 23 From<PathBuf> for  RawPath
rs   impl    ./cymbal/src/utils.rs 19 23 From<RawPath> for  PathBuf
rs   const   ./cymbal/tests/languages/rust.rs 4 11  GLOBAL_COUNTER
...
```

The output columns are
```
language kind path line column leading symbol trailing
```

## Install
Install with `cargo install cymbal` or via the `flake.nix`.


## Use-Case: Jump to symbol from command-line
A potential use of `cymbal` is to jump to symbols from the command-line:
[![asciicast](https://asciinema.org/a/MzqFoRPvOqTztcuUg1PGWnUup.svg)][1]

This was done using the following fish functions:
```fish
function symbol-search -d "shows an fzf menu with symbols at the current directory"
  cymbal --delimiter \u0c --separator0 | fzf \
    --delimiter \u0c \
    --read0 \
    --ansi \
    --preview='bat {3} --color always --style=numbers,snip,header --highlight-line {4} --line-range {4}:+100' \
    --reverse \
    --with-nth='{1} {2} {6,7,8}' \
    --nth=2 \
    --with-shell='bash -c' \
    --bind=tab:down \
    --bind=shift-tab:up \
    --bind='ctrl-r:transform-prompt(
      if [ "$FZF_PROMPT" = "full> " ]; then
        echo "> "
      else
        echo "full> "
      fi
     )+transform-nth(
      if [ "$FZF_PROMPT" = "full> " ]; then
        echo "1.."
      else
        echo "2"
      fi
     )' | awk -v FS=\u0c '{ print $3, $4, $5 }'
end

function symbol-search-open -d "opens a kak instance after a symbol search"
  set symbol (symbol-search $argv | tr ' ' \n )

  if [ -n "$symbol" ]
    kak $symbol[1..-3] +$symbol[-2]:$symbol[-1]
  end
end
```

This example sets up `<c-r>` as a toggle within [fzf][2] to filter for the
entire symbol including leading and trailing text.

## Usage (`cymbal -h`)
```
search for symbols in a codebase

Usage: cymbal [OPTIONS] [SEARCH_PATH]

Arguments:
  [SEARCH_PATH]
          The file or directory to search for symbols in.

          If this is a directory, it is recursively searched for files with supported extensions.

          If this is a file, it is searched for symbols, and the `--language` flag is ignored, and the language appropriate for the file is used.

          [default: .]

Options:
  -c, --config <CONFIG_PATH>
          A toml file with language queries and symbols.

          The default configuration will be applied if this argument is not provided.

  -d, --delimiter <DELIMITER>
          The characters between properties of a single symbol.

          This is the character between the file path, location, kind, text, and leading/trailing text written to stdout.

          This defaults to a space.

          [default: " "]

      --delimiter0
          Set `delimiter` to the null byte. This overrides any `delimiter` value

  -s, --separator <SEPARATOR>
          The character between symbols.

          This defaults to a newline.

          [default: "\n"]

      --separator0
          Set `separator` to the null byte. This overrides any `separator` value

      --language <LANGUAGE>
          Only show symbols from files with extensions matching this language.

          This flag takes precedence over the `--extension` flag.

          [possible values: c, cpp, fish, go, haskell, json, ocaml, odin, python, rust, javascript, tsx, ivy, vine, kak, nu]

      --extension <EXTENSION>
          Only show symbols from files with the language matching this extension.

          The `--language` flag takes precedence over this flag.

      --cache <CACHE_DIRPATH>
          Directory to cache parsed symbols.

          Files are reparsed if their cached mtime differs from than their current mtime, or the path of the file doesn't exist in the cache. This option is typically used when `symbols` is called from the same directory multiple times, such as searching over a code base in an editor.

      --color <COLOR>
          Whether to emit ANSI color escape sequences.

          If the `NO_COLOR` environment variable is set, no ANSI color escape sequences will be emitted if `--color=auto`.

          [default: auto]
          [possible values: never, always, auto]

      --concurrency <CONCURRENCY>
          The number of parser tasks, or roughly the amount of parallelism

      --buffer <CHANNEL_BOUND>
          The maximum number of files to enqueue at any given time.

          Set to 0 to use an unbounded channel.

          [default: 256]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Configuration
`cymbal` is configured with [TOML][3] on a per-language basis. See
[default-config.toml][4] for the default configuration, and
[example-config.toml][5] for an example configuration.

Each language has a set of queries for different kinds of symbols that can be
found in that language. For example for C++,
```toml
[cpp]
type = [
  '(type_definition declarator: (type_identifier) @symbol)',
  '(enum_specifier name: (type_identifier) @symbol)',
]
class = [
  '(struct_specifier name: (type_identifier) @symbol body:(_))',
  '(declaration type: (union_specifier name: (type_identifier) @symbol))',
  '(class_specifier name: (type_identifier) @symbol)',
]
function = '(function_declarator declarator: (identifier) @symbol)'
method = [
  { leading = '{scope}::', query = '(function_declarator declarator: (qualified_identifier scope: (_) @scope name: (identifier) @symbol))' },
  { leading = '{scope.1}::{scope.2}::', query = '(function_declarator declarator: (qualified_identifier scope: (_) @scope.1 name: (qualified_identifier scope: (_) @scope.2 name: (identifier) @symbol)))' },
]
```
There is a fixed set of symbols that are valid, see [`symbol.rs`][6]. For each
language, each symbol kind can have multiple queries, such as `method` above.
For symbol kinds where only a single query is needed, a string can be used,
like in `function` above.

Additionally, each query can be garnished with a `leading` and/or `trailing`
text. These are templates that are hydrated using captures from the tree-sitter
query, such as in the queries for `method` above.

Lastly, the order of the symbols also indicates their priority. `cymbal` will
only emit one symbol per position per file. That is if two queries match at the
same byte in a file, only the query appearing earlier in the configuration will
be emitted. This is useful, for example, for capturing methods along with their
class or struct as context, instead of capturing them as top-level functions.
See the rust `method` query for an example.

### Extending the Default Configuration
To modify just a part of the default configuration, use the `[inherit]` key:
```toml
[inherit]
all = true
# or specify specific languages
languages = ["rust", "python", "c"]

[rust]
function = "some tree-sitter query"
```
When using `[inherit]`, any provided language queries will take precedence over
the inherited ones, however the inherited ones will still be present. That is,
if a symbol matches a provided query, any inherited queries matching that same
symbol in that same position will not emit an entry. Because of this, you can
use the `[inherit]` key to reorder the queries in the default config. For
example, here we give priority to the default `rust.function` queries over the
`rust.method` queries, even though in the default config they are in the
opposite order.
```toml
[inherit]
all = true

[rust]
function = []
method = []
```

## Testing
Snapshot tests live in `cymbal/tests/snapshots.rs`. Each language has a small
sample source file under `cymbal/tests/languages/` (e.g. `cpp.cpp`, `rust.rs`)
and a corresponding snapshot under `cymbal/tests/snapshots/` (e.g.
`cpp.cpp.snap`).

The test harness discovers all files in `cymbal/tests/languages/` automatically.

- `cargo test`: verifies that the current output matches every snapshot.
- `UPDATE_SNAPSHOTS=1 cargo test`: updates snapshot files.


[1]: https://asciinema.org/a/MzqFoRPvOqTztcuUg1PGWnUup
[2]: https://github.com/junegunn/fzf
[3]: https://toml.io/en/
[4]: ./cymbal/default-config.toml
[5]: ./cymbal/example-config.toml
[6]: ./cymbal/src/symbol.rs
