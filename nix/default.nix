inputs: final: prev:
let
  lib = import ./lib.nix inputs { pkgs = final; };

  inherit (prev) callPackage mkShell;
  inherit (prev.lib) optionals;
  inherit (prev.stdenv) isDarwin isLinux;
  inherit (prev.darwin.apple_sdk.frameworks) SystemConfiguration;

  checks = { };
  scripts = callPackage ./scripts.nix { };

  packages = {
    mugraph-node = callPackage ../node/package.nix { };
    mugraph-simulator = callPackage ../simulator/package.nix { };
    default = packages.mugraph-simulator;
  };

  formatter =
    (inputs.treefmt-nix.lib.evalModule prev {
      projectRootFile = "flake.nix";

      settings = {
        allow-missing-formatter = true;
        verbose = 0;

        global.excludes = [
          "*.lock"
          "*.yaml"
        ];

        formatter = {
          nixfmt.options = [ "--strict" ];
          rustfmt.package = lib.rust;
        };
      };

      programs = {
        nixfmt.enable = true;
        oxfmt.enable = true;
        rustfmt.enable = true;
        taplo.enable = true;
      };
    }).config.build.wrapper;

  devShells.default = mkShell {
    inherit (lib.env) RUST_LOG RUSTFLAGS;

    name = "mu-shell";

    packages = [
      lib.rust
      scripts

      prev.aiken
      prev.bun
      prev.cargo-machete
      prev.cargo-nextest
      prev.cargo-pgo
      prev.cargo-tauri
      prev.cargo-watch
      prev.openssl
      prev.pkg-config
      prev.protobuf
      prev.samply
    ]
    ++ optionals isLinux [
      prev.glib
      prev.gtk3
      prev.librsvg
      prev.libsoup_3
      prev.webkitgtk_4_1
    ]
    ++ optionals isDarwin [ SystemConfiguration ];
  };
in
{
  mugraph = {
    inherit
      checks
      devShells
      formatter
      inputs
      lib
      packages
      ;
  };
}
