{
  cargo-risczero,
  darwin,
  lib,
  mugraph,
  openssl,
  pkg-config,
  stdenv,
  risc0-rust,
}:
extraBuildRustPackageAttrs@{
  nativeBuildInputs ? [ ],
  buildInputs ? [ ],
  ...
}:

let
  inherit (mugraph) rdt rustup-mock;
  inherit (lib) recursiveUpdate unique optionals;
  inherit (stdenv) isDarwin;

  extraBuildRustPackageAttrsNoArgs = builtins.removeAttrs extraBuildRustPackageAttrs [
    "buildInputs"
    "nativeBuildInputs"
    "preBuild"
  ];
in

rdt.buildRustPackage (
  recursiveUpdate extraBuildRustPackageAttrsNoArgs {
    nativeBuildInputs = unique (
      [
        rustup-mock
        cargo-risczero
        pkg-config
      ]
      ++ nativeBuildInputs
    );

    env.RISC0_RUST_SRC = "${risc0-rust}/lib/rustlib/src/rust";

    buildInputs = unique (
      [ openssl.dev ]
      ++ optionals isDarwin [ darwin.apple_sdk.frameworks.SystemConfiguration ]
      ++ buildInputs
    );

    doCheck = false;
    auditable = false;
  }
)
