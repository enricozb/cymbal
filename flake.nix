{
  description = "lists symbols (types, classes, etc.) in a codebase";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    fenix = {
      url = "github:nix-community/fenix/monthly";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    trix = {
      url = "github:enricozb/trix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    tree-sitter-c = {
      url = "github:tree-sitter/tree-sitter-c";
      flake = false;
    };
    tree-sitter-cpp = {
      url = "github:tree-sitter/tree-sitter-cpp";
      flake = false;
    };
    tree-sitter-fish = {
      url = "github:ram02z/tree-sitter-fish";
      flake = false;
    };
    tree-sitter-go = {
      url = "github:tree-sitter/tree-sitter-go";
      flake = false;
    };
    tree-sitter-haskell = {
      url = "github:tree-sitter/tree-sitter-haskell";
      flake = false;
    };
    tree-sitter-ocaml = {
      url = "github:tree-sitter/tree-sitter-ocaml";
      flake = false;
    };
    tree-sitter-odin = {
      url = "github:tree-sitter-grammars/tree-sitter-odin";
      flake = false;
    };
    tree-sitter-python = {
      url = "github:tree-sitter/tree-sitter-python";
      flake = false;
    };
    tree-sitter-rust = {
      url = "github:tree-sitter/tree-sitter-rust";
      flake = false;
    };
    tree-sitter-javascript = {
      url = "github:tree-sitter/tree-sitter-javascript";
      flake = false;
    };
    tree-sitter-typescript = {
      url = "github:tree-sitter/tree-sitter-typescript";
      flake = false;
    };
    tree-sitter-json = {
      url = "github:tree-sitter/tree-sitter-json";
      flake = false;
    };
    tree-sitter-vine = {
      url = "github:VineLang/vine";
      flake = false;
    };
    tree-sitter-kak = {
      url = "github:saifulapm/tree-sitter-kakscript";
      flake = false;
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      crane,
      fenix,

      trix,
      tree-sitter-c,
      tree-sitter-cpp,
      tree-sitter-fish,
      tree-sitter-go,
      tree-sitter-haskell,
      tree-sitter-javascript,
      tree-sitter-json,
      tree-sitter-kak,
      tree-sitter-ocaml,
      tree-sitter-odin,
      tree-sitter-python,
      tree-sitter-rust,
      tree-sitter-typescript,
      tree-sitter-vine,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        rust-toolchain = fenix.packages.${system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-tqagmXrHoZA9Zmu2Br6n3MzvXaLkuPzKPS3NIVdNQVQ=";
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain (_: rust-toolchain);
        cymbalSrc =
          let
            extraFilter =
              path:
              nixpkgs.lib.any (suffix: nixpkgs.lib.hasSuffix suffix path) [
                ".js"
                ".sql"
              ];
          in
          nixpkgs.lib.cleanSourceWith {
            src = ./.;
            filter = path: type: (craneLib.filterCargoSources path type) || (extraFilter path);
          };
        grammar-srcs = {
          c.src = tree-sitter-c;
          cpp.src = tree-sitter-cpp;
          fish.src = tree-sitter-fish;
          go.src = tree-sitter-go;
          haskell.src = tree-sitter-haskell;
          ocaml = {
            src = tree-sitter-ocaml;
            filter = [ "ocaml" ];
          };
          odin.src = tree-sitter-odin;
          python.src = tree-sitter-python;
          rust.src = tree-sitter-rust;
          javascript.src = tree-sitter-javascript;
          typescript = {
            src = tree-sitter-typescript;
            filter = [
              "typescript"
              "tsx"
            ];
          };
          json.src = tree-sitter-json;
          ivy.src = "${tree-sitter-vine}/lsp/tree-sitter-ivy";
          vine.src = "${tree-sitter-vine}/lsp/tree-sitter-vine";
          kak.src = tree-sitter-kak;
        };
        grammars = trix.mkTrixConfig.${system} grammar-srcs;
      in
      {
        packages.default =
          let
            cargo-toml = craneLib.crateNameFromCargoToml { cargoToml = ./cymbal/Cargo.toml; };
          in
          craneLib.buildPackage {
            inherit (cargo-toml) pname version;
            src = cymbalSrc;
            env.TRIX_CONFIG = grammars;
          };

        devShells.default = craneLib.devShell {
          TRIX_CONFIG = grammars;
          packages = [
            pkgs.cargo-expand
            pkgs.nodejs_24
            pkgs.tree-sitter
          ];
        };

        formatter = pkgs.nixfmt-tree;
      }
    );
}
