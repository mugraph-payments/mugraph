inputs: final: prev:
let
  lib = import ./lib.nix { pkgs = final; };

  inherit (builtins) concatStringsSep;
  inherit (lib) buildPackageSet;
  inherit (prev) mkShell;
  inherit (prev.lib) optionals;
  inherit (prev.stdenv) isDarwin;

  dependencies = buildPackageSet ./dependencies;
  checks = buildPackageSet ./checks;

  devShells.default = mkShell {
    name = "mu-shell";

    packages = [
      lib.defaults.rust
      checks.pre-commit.enabledPackages

      final.rustup
      final.cargo-watch
      final.cargo-nextest
    ] ++ optionals isDarwin [ final.darwin.apple_sdk.frameworks.SystemConfiguration ];

    inherit (lib.defaults.env) RUST_LOG RISC0_RUST_SRC RUSTFLAGS;

    RISC0_PROVER = "local";
    RISC0_EXECUTOR = "local";

    shellHook = concatStringsSep "\n\n" [
      checks.pre-commit.shellHook
      ''
        rustup toolchain link mugraph ${lib.defaults.rust}
        rustup toolchain link risc0 ${lib.defaults.rust}
        rustup override set mugraph
      ''
    ];
  };
in
{
  mugraph = {
    inherit
      checks
      devShells
      lib
      inputs
      dependencies
      ;
  };
}
