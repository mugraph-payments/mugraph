{
  mugraph,
  makeWrapper,
  lib,
}:
let
  inherit (lib) makeBinPath;
  inherit (mugraph.dependencies)
    rustPlatform
    rust
    r0vm
    rustup-mock
    ;
in
rustPlatform.buildRustPackage {
  pname = "mugraph-node";
  version = "0.0.1";
  src = ../..;

  env.RISC0_RUST_SRC = "${rust}/lib/rustlib/src/rust";

  cargoLock.lockFile = ../../Cargo.lock;
  nativeBuildInputs = [
    makeWrapper
    rustup-mock
  ];

  postInstall = ''
    wrapProgram $out/bin/host \
      --set PATH ${makeBinPath [ r0vm ]}
  '';
}
