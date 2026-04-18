provide-module my-module %{
  # These are not seen (yet) since blocks are always treated opaquely by the
  # kakoune tree-sitter grammar.

  define-command my-command1 %{
  }

  define-command my-command2 %{
  }

  define-command my-command3 %{
  }

  declare-user-mode lsp

  hook global KakEnd .* %{
  }
}

define-command my-command -params 1 %{
}

declare-user-mode my-user-mode

hook global KakBegin .* %{
}
