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

  dependencies = listToAttrs (
    map (file: {
      name = elemAt (match "(.*)\\.nix" file) 0;
      value = final.callPackage (./dependencies + "/${file}") { };
    }) (attrNames (readDir ./dependencies))
  );

  rust = final.symlinkJoin {
    inherit (final.rust-bin.nightly.latest.complete) meta;

    name = "mugraph-rustc";
    paths = [
      (final.rust-bin.nightly.latest.complete)
      dependencies.risc0-rust
    ];
  };

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

  devShells.default = mkShell {
    name = "mu-shell";

    packages = [
      rust
      (attrValues dependencies)
      final.cargo-nextest
      final.cargo-watch
    ];

    RISC0_DEV_MODE = 1;
    RISC0_RUST_SRC = "${rust}/lib/rustlib/src/rust";
  };
in
{
  mugraph = {
    inherit devShells inputs packages;

    dependencies = dependencies // {
      inherit rust rustPlatform;
    };
  };
}
