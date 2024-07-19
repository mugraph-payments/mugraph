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

        inherit (pkgs) mkShell mugraph makeWrapper;
        inherit (pkgs.lib) makeBinPath;

        package = mugraph.buildRisc0Package {
          pname = "risc0package";
          version = "0.0.1";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = [ makeWrapper ];
          postInstall = ''
            wrapProgram $out/bin/host \
              --set PATH ${makeBinPath [ mugraph.r0vm ]}
          '';
        };
      in
      {
        packages.default = package;

        devShells.default = mkShell {
          RISC0_RUST_SRC = "${package.toolchain}/lib/rustlib/src/rust";
          RISC0_DEV_MODE = 1;

          inputsFrom = [
            mugraph.rdt.devShell
            package
          ];

          packages = [
            mugraph.r0vm
            pkgs.iconv
          ];
        };
      }
    );
}
