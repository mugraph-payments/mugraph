{
  mugraph,
  makeWrapper,
  lib,
  stdenv,
  darwin,
}:
let
  inherit (stdenv) isDarwin;
  inherit (lib) optionals;
  inherit (mugraph) rustPlatform;
in
rustPlatform.buildRustPackage {
  pname = "mugraph-node";
  version = "0.0.1";
  src = ../..;

  cargoLock = {
    lockFile = ../../Cargo.lock;
  };

  buildInputs = [
    makeWrapper
  ] ++ optionals isDarwin [ darwin.apple_sdk.frameworks.SystemConfiguration ];

  useNextest = true;

  buildAndTestSubdir = "core";

  postInstall = ''
    wrapProgram $out/bin/mugraph-node \
      --set RUST_LOG info
  '';
}
