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

  cargoLock.lockFile = ../../Cargo.lock;

  buildInputs = optionals isDarwin [ darwin.apple_sdk.frameworks.SystemConfiguration ];

  useNextest = true;

  postInstall = ''
    ${makeWrapper}/bin/wrapProgram $out/bin/mugraph-node \
      --set RUST_LOG info
  '';
}
