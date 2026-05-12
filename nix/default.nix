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

  # mold/lld linker flags, scoped to the host target rather than a global
  # RUSTFLAGS — a global one would override the NDK linker that `cargo tauri
  # android` sets up for the *-linux-android targets and break those builds.
  hostRustflagsEnv =
    "CARGO_TARGET_"
    + prev.lib.toUpper (
      builtins.replaceStrings [ "-" "." ] [ "_" "_" ] prev.stdenv.hostPlatform.rust.rustcTarget
    )
    + "_RUSTFLAGS";

  devShells.default = mkShell {
    inherit (lib.env) RUST_LOG;
    # JDK + Android SDK/NDK so `cargo tauri android {init,dev,build}` works.
    inherit (lib.androidEnv)
      JAVA_HOME
      ANDROID_HOME
      ANDROID_SDK_ROOT
      ANDROID_NDK_ROOT
      ANDROID_NDK_HOME
      NDK_HOME
      GRADLE_OPTS
      ;
    ${hostRustflagsEnv} = lib.env.RUSTFLAGS;

    name = "mu-shell";

    packages = [
      lib.androidRust
      prev.jdk17
      lib.androidComposition.androidsdk

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
