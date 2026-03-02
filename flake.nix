{
  description = "lists symbols (types, classes, etc.) in a codebase";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.crane = {
    url = "github:ipetkov/crane";
    inputs.nixpkgs.follows = "nixpkgs";
  };
  inputs.fenix = {
    url = "github:nix-community/fenix/monthly";
    inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      crane,
      fenix,
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
      in
      {
        packages.default =
          let
            cargo-toml = craneLib.crateNameFromCargoToml { cargoToml = ./cymbal/Cargo.toml; };
          in
          craneLib.buildPackage {
            inherit (cargo-toml) pname version;
            src = ./.;
            strictDeps = true;
          };

        devShells.default = craneLib.devShell {
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
