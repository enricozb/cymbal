{
  description = "symbols";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.fenix.url = "github:nix-community/fenix/monthly";
  inputs.flake-compat.url = "https://flakehub.com/f/edolstra/flake-compat/1.tar.gz";

  outputs = { self, nixpkgs, flake-utils, fenix, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        rust-toolchain = fenix.packages.${system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-+9FmLhAOezBZCOziO0Qct1NOrfpjNsXxc/8I0c7BdKE=";
        };
        rust-platform = pkgs.makeRustPlatform {
          cargo = rust-toolchain;
          rustc = rust-toolchain;
        };
      in {
        devShells.default = pkgs.mkShell { packages = [ rust-toolchain pkgs.cargo-flamegraph ]; };

        packages.default = rust-platform.buildRustPackage {
          pname = "cymbal";
          version = "0.8.6";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              "tree-sitter-fish-3.5.1" = "sha256-ED1lJ1GlbT/ptr+S9J1mD9fnfuewPko2rvj/qMVPCso=";
            };
          };
        };
      });
}
