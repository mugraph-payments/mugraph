{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";

    rust-dev-tools = {
      url = "github:cfcosta/rust-dev-tools.nix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs =
    {
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
          overlays = [ rust-dev-tools.overlays.default ];
        };

        rdt = rust-dev-tools.setup pkgs {
          name = "mugraph";
          rust = rust-dev-tools.version.fromToolchainFile ./rust-toolchain.toml;
          dependencies = with pkgs; [ ];
        };
      in
      {
        devShells.default = pkgs.mkShell { inputsFrom = [ rdt.devShell ]; };
      }
    );
}
