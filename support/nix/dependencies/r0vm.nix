{
  darwin,
  mugraph,
  perl,
  pkg-config,
  stdenv,
  lib,
}:
let
  inherit (builtins) fetchurl;
  inherit (darwin.apple_sdk) frameworks;
  inherit (lib) optionals;
  inherit (mugraph.lib.defaults) rustPlatform rust;
  inherit (mugraph.inputs) risc0;
  inherit (stdenv) isDarwin;

  # see https://github.com/risc0/risc0/blob/main/risc0/circuit/recursion/build.rs
  sha256Hash = "4e8496469e1efa00efb3630d261abf345e6b2905fb64b4f3a297be88ebdf83d2";

  recursionZkr = fetchurl {
    name = "recursion_zkr.zip";
    url = "https://risc0-artifacts.s3.us-west-2.amazonaws.com/zkr/${sha256Hash}.zip";
    sha256 = "sha256:1ll3vzmqiglplbrv8r7v0llnnpilpwd2c3b3ngph1yhykr39d12f";
  };
in
rustPlatform.buildRustPackage {
  pname = "r0vm";
  version = "1.0.5";
  src = risc0;

  buildAndTestSubdir = "risc0/r0vm";

  env = {
    RISC0_RUST_SRC = "${rust}/lib/rustlib/src/rust";
    RECURSION_SRC_PATH = recursionZkr;
  };

  nativeBuildInputs = [
    pkg-config
    perl
  ] ++ optionals isDarwin [ frameworks.SystemConfiguration ];

  doCheck = false;
  cargoLock.lockFile = "${risc0}/Cargo.lock";

  meta = {
    homepage = "https://github.com/risc0/risc0";
    description = "risc0's zkVM";
  };
}
