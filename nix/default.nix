inputs: final: prev:
let
  inherit (builtins)
    attrNames
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

  rust = final.symlinkJoin {
    inherit (final.rust-bin.nightly.latest.complete) meta;

    name = "mugraph-rustc";
    paths = [
      (final.rust-bin.nightly.latest.complete)
      packages.risc0-rust
    ];
  };

  rustPlatform = final.makeRustPlatform {
    rustc = rust;
    cargo = rust;
  };

  devShells.default = mkShell {
    name = "mu-shell";

    packages = [
      rust

      packages.cargo-risczero
      packages.r0vm
      packages.rustup-mock

      final.cargo-nextest
      final.cargo-watch
    ];

    RISC0_DEV_MODE = 1;
    RISC0_RUST_SRC = "${rust}/lib/rustlib/src/rust";
  };
in
{
  mugraph = {
    inherit
      devShells
      inputs
      packages
      rust
      rustPlatform
      ;
  };
}
