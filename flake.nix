{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    rust-dev-tools = {
      url = "github:cfcosta/rust-dev-tools.nix";
      inputs = {
        flake-utils.follows = "flake-utils";
        nixpkgs.follows = "nixpkgs";
        rust-overlay.follows = "rust-overlay";
      };
    };

    risc0 = {
      url = "github:risc0/risc0/v1.0.3";
      flake = false;
    };
  };

  outputs =
    inputs@{
      nixpkgs,
      flake-utils,
      rust-dev-tools,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            rust-dev-tools.overlays.default
            (import ./nix inputs)
          ];
        };
      in
      {
        devShells.default =
          with pkgs;
          mkShell {
            inputsFrom = [ mugraph.rdt.devShell ];
            packages = [
              iconv
              mugraph.r0vm
              cargo-risczero
            ];
          };
      }
    );
}
