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
      prev.protobuf
      prev.samply
      scripts
    ] ++ optionals isDarwin [ SystemConfiguration ];
  };
in
{
  mugraph = {
    inherit
      checks
      devShells
      inputs
      lib
      packages
      ;
  };
}
