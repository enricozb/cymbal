{ pkgs ? import <nixpkgs> { } }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "cymbal";
  version = "0.8.0";
  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = { "tree-sitter-fish-3.5.1" = "sha256-ED1lJ1GlbT/ptr+S9J1mD9fnfuewPko2rvj/qMVPCso="; };
  };
}
