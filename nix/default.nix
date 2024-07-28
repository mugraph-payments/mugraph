inputs: final: prev:
let
  inherit (builtins)
    attrNames
    elemAt
    listToAttrs
    match
    readDir
    ;

  inherit (prev) mkShell callPackage;
  inherit (final.stdenv) isLinux;
  inherit (final.lib) optionals optionalAttrs;

  rust = callPackage ./rust { };

  rustPlatform = final.makeRustPlatform {
    rustc = rust;
    cargo = rust;
  };

  packages = listToAttrs (
    map (file: {
      name = elemAt (match "(.*)\\.nix" file) 0;
      value = final.callPackage (./packages + "/${file}") { };
    }) (attrNames (readDir ./packages))
  );

  checks.pre-commit = inputs.pre-commit-hooks.lib.${final.system}.run {
    src = ../.;
    hooks = {
      nixfmt = {
        enable = true;
        package = final.nixfmt-rfc-style;
      };

      rustfmt = {
        enable = true;
        packageOverrides = {
          cargo = rust;
          rustfmt = rust;
        };
      };
    };
  };

  devShells.default =
    mkShell {
      inherit (checks.pre-commit) shellHook;
      inherit (rust) RUSTFLAGS;

      name = "mu-shell";

      packages = [
        rust
        checks.pre-commit.enabledPackages

        final.cargo-nextest
        final.cargo-watch
      ] ++ optionals isLinux [ packages.r0vm ];

      RISC0_RUST_SRC = "${rust}/lib/rustlib/src/rust";
      RUST_LOG = "info";
    }
    // optionalAttrs isLinux {
      RISC0_EXECUTOR = "ipc";
      RISC0_SERVER_PATH = "${packages.r0vm}/bin/r0vm";
    };
in
{
  mugraph = {
    inherit
      devShells
      inputs
      checks
      rust
      rustPlatform
      ;

    packages = packages // {
      inherit rust;

      default = packages.mugraph-node;
    };
  };
}
