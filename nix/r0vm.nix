{
  rdt,
  risc0-source,
  stdenv,
  pkg-config,
  perl,
  openssl,
  lib,
  darwin,
}:
let
  inherit (builtins) fetchurl;
  inherit (lib) optionals;
  inherit (stdenv) isDarwin;
in
rdt.buildRustPackage {
  pname = "r0vm";
  version = "1.0.3";
  src = risc0-source;

  buildAndTestSubdir = "risc0/r0vm";

  nativeBuildInputs = [
    pkg-config
    perl
  ];

  buildInputs = [
    openssl.dev
  ] ++ optionals isDarwin [ darwin.apple_sdk.frameworks.SystemConfiguration ];

  doCheck = false;
  cargoLock.lockFile = "${risc0-source}/Cargo.lock";

  postPatch =
    let
      # see https://github.com/risc0/risc0/blob/main/risc0/circuit/recursion/build.rs
      sha256Hash = "4e8496469e1efa00efb3630d261abf345e6b2905fb64b4f3a297be88ebdf83d2";

      recursionZkr = fetchurl {
        name = "recursion_zkr.zip";
        url = "https://risc0-artifacts.s3.us-west-2.amazonaws.com/zkr/${sha256Hash}.zip";
        sha256 = "sha256:08zcl515890gyivhj8rgyi72q0qcr515bbm1vrsbkb164raa411m";
      };
    in
    "cp ${recursionZkr} ./risc0/circuit/recursion/src/recursion_zkr.zip";

  meta = {
    homepage = "https://github.com/risc0/risc0";
    description = "risc0's zkVM";
  };
}
