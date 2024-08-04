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

    packages =
      [
        lib.defaults.rust
        checks.pre-commit.enabledPackages

        final.cargo-nextest
        final.cargo-watch
        final.rustup

        dependencies.r0vm
      ]
      ++ optionals isDarwin [
        final.darwin.apple_sdk.frameworks.SystemConfiguration
        final.darwin.apple_sdk.frameworks.Metal
        final.darwin.apple_sdk.frameworks.CoreGraphics
      ];

    inherit (lib.defaults.env) RUST_LOG RISC0_RUST_SRC RUSTFLAGS;

    RISC0_PROVER = "ipc";
    RISC0_EXECUTOR = "ipc";

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
