{ pkgs ? import <nixpkgs> { } }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "cymbal";
  version = "0.2.0";
  src = ./.;

  cargoLock = { lockFile = ./Cargo.lock; };
}
