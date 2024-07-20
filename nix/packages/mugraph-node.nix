{
  mugraph,
  makeWrapper,
  lib,
}:
let
  inherit (lib) makeBinPath;
in
mugraph.risc0Platform.buildRustPackage {
  pname = "mugraph-node";
  version = "0.0.1";
  src = ../..;

  cargoLock.lockFile = ../../Cargo.lock;
  nativeBuildInputs = [ makeWrapper ];

  postInstall = ''
    wrapProgram $out/bin/host \
      --set PATH ${makeBinPath [ mugraph.packages.r0vm ]}
  '';
}
