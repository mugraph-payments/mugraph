inputs: final: prev:
let
  inherit (builtins)
    attrNames
    attrValues
    elemAt
    listToAttrs
    match
    readDir
    ;

  inherit (prev) mkShell;

  packages = listToAttrs (
    map (file: {
      name = elemAt (match "(.*)\\.nix" file) 0;
      value = final.callPackage (./packages + "/${file}") { };
    }) (attrNames (readDir ./packages))
  );

  risc0Platform = final.makeRustPlatform {
    rustc = packages.risc0-rust;
    cargo = packages.risc0-rust;
  };

  rust = final.rust-bin.nightly.latest.complete.override {
    extensions = [
      "rust-src"
      "clippy"
      "rustfmt"
      "rust-analyzer"
    ];
  };

  rustPlatform = final.makeRustPlatform {
    rustc = rust;
    cargo = rust;
  };

  devShells.default = mkShell {
    name = "mu-shell";

    packages = (attrValues packages) ++ [
      final.cargo-nextest
      final.cargo-watch
    ];

    RISC0_DEV_MODE = 1;
  };
in
{
  mugraph = {
    inherit
      devShells
      inputs
      packages
      risc0Platform
      rust

      rustPlatform

      ;
  };
}
