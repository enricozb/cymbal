# cymbal - list symbols in a codebase

## Overview
`cymbal` lists all symbols in a codebase. For example,
```
$ cymbal --delimiter ' ' --separator \n
(rs) (impl)   ./cymbal/src/ext.rs 10 20  ResultExt
(rs) (impl)   ./cymbal/src/ext.rs 31 20  OptionExt
(rs) (enum)   ./cymbal/src/config.rs 19 10  Language
(rs) (func)   ./cymbal/src/args.rs 82 6  raw_config
(rs) (func)   ./cymbal/src/args.rs 140 10  num_threads
(rs) (struct) ./cymbal/src/symbol.rs 6 12  Symbol
(rs) (func)   ./cymbal/src/text.rs 10 10  new
(rs) (struct) ./cymbal/src/cache.rs 16 12  Cache
(rs) (struct) ./cymbal/src/cache.rs 22 12  FileInfo
(rs) (struct) ./cymbal/src/text.rs 16 12  Span
(rs) (impl)   ./cymbal/src/text.rs 9 6  Loc
...
```

## Use-Case: Jump to symbol from command-line
A potential use of `cymbal` is to jump to symbols from the command-line:
[![asciicast](https://asciinema.org/a/MzqFoRPvOqTztcuUg1PGWnUup.svg)][1]

This was done using the following fish functions:
```fish
function symbol-search -d "shows an fzf menu with symbols at the current directory"
  cymbal --delimiter \u0c | fzf \
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

## Usage
```
Usage: cymbal [OPTIONS] [PATH]

Arguments:
  [PATH]
          [default: .]

Options:
  -c, --config <CONFIG>
          Language configurations.

          This can either be a path to a .toml file or a TOML string.

          The default configuration will be applied if this argument is not provided or if it is the empty string.

      --cache-dir <CACHE_DIR>
          Directory to cache parsed symbols.

          Files are reparsed if their cached mtime differs from than their current mtime, or the path of the file doesn't exist in the cache. This option is typically used when `symbols` is called from the same directory multiple times, such as searching over a code base in an editor.

          The directory is created if it does not exist.

  -d, --delimiter <DELIMITER>
          The characters between properties of a single symbol.

          This is the character between the path, location, kind, text, and leading/trailing text written to stdout.

          This defaults to U+200B (zero-width space).

          [default: ​]

  -s, --separator <SEPARATOR>
          The character between symbols.

          This defaults to the U+0 (null byte).

          [default: ]

      --detached
          Whether to spawn a detached process to index symbols.

          Only useful with the `cache-dir` option.

          If this option is false, the cache may not be created if the process is exited prematurely. This can happen if using `symbols` in a pipeline (such as with `fzf`) and selecting a symbol before indexing is complete.

          If this option is true, indexing is performed by a separate detached process whose output is redirected to stdout. Then, if `symbols` is exited prematurely, the indexing will still be able to complete.

  -t, --threads <THREADS>
          The number of worker threads to use when parsing files.

          This defaults to `std::thread::available_parallelism` if it is available, and otherwise defaults to 8.

  -l, --language <LANGUAGE>
          Only show symbols from files with extensions matching this language.

          This option takes precedence over `--extension`.

          [possible values: c, cpp, go, haskell, odin, python, rust, type-script]

  -e, --extension <EXTENSION>
          Only show symbols from files with extensions matching this extension's language. Note that this will not filter for symbols in files matching this extension, but for files with the same language as this extension's.

          The `--language` option takes precedence over `--extension`.

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

[1]: https://asciinema.org/a/MzqFoRPvOqTztcuUg1PGWnUup
[2]: https://github.com/junegunn/fzf
[3]: https://toml.io/en/
[4]: ./cymbal/default-config.toml
[5]: ./cymbal/example-config.toml
[6]: ./cymbal/src/symbol.rs
