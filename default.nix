{ pkgs ? import <nixpkgs> { } }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "cymbal";
  version = "0.3.1";
  src = ./.;

  cargoLock = { lockFile = ./Cargo.lock; };
}
