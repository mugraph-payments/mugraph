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
      pname = "mu-fix";
      name = "µ-fix";
      runtimeInputs = [ rust ];
      text = "${rust}/bin/cargo clippy --fix --allow-dirty";
    };

    fmt = writeShellApplication {
      pname = "mu-fix";
      name = "µ-fmt";
      runtimeInputs = [ rust ];
      text = "exec ${rust}/bin/cargo fmt $@";
    };

    test = writeShellApplication {
      pname = "mu-fix";
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
