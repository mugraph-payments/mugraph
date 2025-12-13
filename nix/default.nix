inputs: final: prev:
let
  lib = import ./lib.nix inputs { pkgs = final; };

  inherit (prev) callPackage mkShell;
  inherit (prev.lib) optionals;
  inherit (prev.stdenv) isDarwin;
  inherit (prev.darwin.apple_sdk.frameworks) SystemConfiguration;

  checks.pre-commit = callPackage ./pre-commit-hook.nix { };
  scripts = callPackage ./scripts.nix { };

  packages = {
    mugraph-node = callPackage ../node/package.nix { };
    mugraph-simulator = callPackage ../simulator/package.nix { };
    default = packages.mugraph-simulator;
  };

  formatter =
    (inputs.treefmt-nix.lib.evalModule prev {
      projectRootFile = "flake.nix";

      settings = {
        allow-missing-formatter = true;
        verbose = 0;

        global.excludes = [ "*.lock" ];

        formatter = {
          nixfmt.options = [ "--strict" ];
          rustfmt.package = lib.rust;
        };
      };

      programs = {
        nixfmt.enable = true;
        taplo.enable = true;
        rustfmt.enable = true;
      };
    }).config.build.wrapper;

  devShells.default = mkShell {
    inherit (lib.env) RUST_LOG RUSTFLAGS;
    inherit (checks.pre-commit) shellHook;

    name = "mu-shell";

    packages = [
      checks.pre-commit.enabledPackages
      lib.rust
      prev.cargo-machete
      prev.cargo-nextest
      prev.cargo-pgo
      prev.cargo-watch
      prev.openssl
      prev.pkg-config
      prev.protobuf
      prev.samply
      prev.typst
      scripts
    ]
    ++ optionals isDarwin [ SystemConfiguration ];
  };
in
{
  mugraph = {
    inherit
      checks
      devShells
      formatter
      inputs
      lib
      packages
      ;
  };
}
