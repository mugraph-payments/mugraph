{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";

    rust-dev-tools = {
      url = "github:cfcosta/rust-dev-tools.nix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };

    leo = {
      url = "github:AleoHQ/leo";
      flake = false;
    };

    snarkvm = {
      url = "github:AleoHQ/snarkvm";
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
            (
              _: _:
              import ./overlay.nix {
                inherit pkgs;
                inherit inputs;
              }
            )
          ];
        };

        rdt = rust-dev-tools.setup pkgs {
          name = "µ";
          rust = rust-dev-tools.version.fromToolchainFile ./rust-toolchain.toml;
          dependencies = with pkgs; [ ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          inputsFrom = [ rdt.devShell ];
          packages = with pkgs; [
            leo
            snarkvm
            b3sum
          ];
        };
      }
    );
}
