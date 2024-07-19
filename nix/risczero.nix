{
  cargo-risczero,
  darwin,
  lib,
  mugraph,
  openssl,
  pkg-config,
  stdenv,
  writeShellApplication,
  toolchain,
}:
extraBuildRustPackageAttrs@{
  nativeBuildInputs ? [ ],
  preBuild ? "",
  buildInputs ? [ ],
  ...
}:

let
  inherit (mugraph) rdt;
  inherit (lib) recursiveUpdate unique;

  rustup-mock = writeShellApplication {
    name = "rustup";
    text = ''
      # the buildscript uses rustup toolchain to check
      # whether the risc0 toolchain was installed
      if [[ "$1" = "toolchain" ]]
      then
        printf "risc0\n"
      elif [[ "$1" = "+risc0" ]]
      then
        printf "${toolchain}/bin/rustc"
      fi
    '';
  };

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
        pkg-config
        cargo-risczero
        rustup-mock
      ]
      ++ nativeBuildInputs
    );

    preBuild = ''
      export RISC0_RUST_SRC=${toolchain}/lib/rustlib/src/rust;
      ${preBuild}
    '';

    buildInputs = unique (
      [ openssl.dev ]
      ++ lib.optionals stdenv.isDarwin [ darwin.apple_sdk.frameworks.SystemConfiguration ]
      ++ buildInputs
    );

    doCheck = false;
    auditable = false;

    passthru = {
      toolchain = toolchain;
    };
  }
)
