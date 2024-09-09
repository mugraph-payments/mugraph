{
  cargo-nextest,
  mugraph,
  symlinkJoin,
  writeShellApplication,
  lib,
  ...
}:
let
  inherit (mugraph.lib) rust;
  inherit (lib) attrValues;

  scripts = {
    fix = writeShellApplication {
      name = "µ-fix";
      runtimeInputs = [ rust ];
      text = "${rust}/bin/cargo clippy --fix --allow-dirty";
    };

    fmt = writeShellApplication {
      name = "µ-fmt";
      runtimeInputs = [ rust ];
      text = "${rust}/bin/cargo fmt";
    };

    test = writeShellApplication {
      name = "µ-test";
      runtimeInputs = [
        rust
        cargo-nextest
      ];
      text = "${rust}/bin/cargo nextest run";
    };
  };
in
symlinkJoin {
  name = "µ-scripts";
  paths = attrValues scripts;
}
