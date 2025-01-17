{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    process-compose.url = "github:Platonic-Systems/process-compose-flake";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      pre-commit-hooks,
      process-compose,
      rust-overlay,
      treefmt-nix,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        inherit (pkgs)
          makeRustPlatform
          mkShell
          rust-bin
          writeShellApplication
          ;

        rust = rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        rustPlatform = makeRustPlatform {
          rustc = rust;
          cargo = rust;
        };

        treefmt =
          (treefmt-nix.lib.evalModule pkgs {
            projectRootFile = "flake.nix";

            settings = {
              allow-missing-formatter = true;
              verbose = 0;

              global.excludes = [ "*.lock" ];

              formatter = {
                nixfmt.options = [ "--strict" ];
                rustfmt.package = rust;
              };
            };

            programs = {
              nixfmt.enable = true;
              taplo.enable = true;
              rustfmt.enable = true;
            };
          }).config.build.wrapper;

        check-and-test = writeShellApplication {
          name = "mugraph-ci";
          runtimeInputs = with pkgs; [
            rust
            cargo-watch
            cargo-nextest
          ];
          text = ''
            export RUSTFLAGS="-C target-cpu=native -C target-feature=+avx2,+avx,+sse2,+avx512f,+avx512bw,+avx512vl"
            cargo watch -s 'cargo clippy && cargo nextest run --release'
          '';
        };

        packages = {
          default = rustPlatform.buildRustPackage {
            name = "mugraph";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            doCheck = false;
          };

          llvm-bolt = pkgs.llvmPackages_19.bolt;

          mugraph-watch = (import process-compose.lib { inherit pkgs; }).makeProcessCompose {
            modules = [ { settings.processes.mugraph-mint.command = "${check-and-test}/bin/mugraph-ci"; } ];
          };
        };

        pre-commit-check = pre-commit-hooks.lib.${system}.run {
          src = ./.;

          hooks = {
            deadnix.enable = true;
            nixfmt-rfc-style.enable = true;
            treefmt = {
              enable = true;
              package = treefmt;
            };
          };
        };
      in
      {
        inherit packages;

        checks = { inherit pre-commit-check; };
        formatter = treefmt;

        devShells.default = mkShell {
          inherit (pre-commit-check) shellHook;

          name = "mugraph";

          buildInputs = with pkgs; [
            packages.mugraph-watch
            packages.llvm-bolt
            rust

            cargo-nextest
            cargo-pgo
            cargo-watch
            cargo-machete
          ];
        };
      }
    );
}
