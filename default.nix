{ pkgs ? import <nixpkgs> { } }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "cymbal";
  version = "0.2.1";
  src = ./.;

  cargoLock = { lockFile = ./Cargo.lock; };
}
