inputs: final: prev:
let
  lib = import ./lib.nix inputs { pkgs = final; };

  inherit (lib) buildPackageSet;
  inherit (prev) mkShell;
  inherit (prev.lib) optionals;
  inherit (prev.stdenv) isDarwin;

  checks = buildPackageSet ./checks;
  packages = buildPackageSet ./packages // {
    default = packages.mugraph-simulator;
  };

  devShells.default = mkShell {
    inherit (lib.defaults.env) RUST_LOG RUSTFLAGS;
    inherit (checks.pre-commit) shellHook;

    name = "mu-shell";

    packages = [
      checks.pre-commit.enabledPackages
      final.cargo-nextest
      final.cargo-watch
      final.protobuf
      final.samply
      lib.defaults.rust
    ] ++ optionals isDarwin [ final.darwin.apple_sdk.frameworks.SystemConfiguration ];
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
