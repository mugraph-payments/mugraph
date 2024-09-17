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
      name = "mu-fix";
      runtimeInputs = [ rust ];
      text = ''
        exec ${rust}/bin/cargo clippy --fix --allow-dirty "$@"
      '';
    };

    fmt = writeShellApplication {
      name = "mu-fmt";
      runtimeInputs = [ rust ];
      text = ''
        exec ${rust}/bin/cargo fmt "$@"
      '';
    };

    test = writeShellApplication {
      name = "mu-test";
      runtimeInputs = [
        rust
        cargo-nextest
      ];
      text = ''
        exec ${rust}/bin/cargo nextest run "$@"
      '';
    };
  };
in
symlinkJoin {
  name = "mu-scripts";
  paths = attrValues scripts;
}
