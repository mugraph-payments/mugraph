{
  mugraph,
  makeWrapper,
  lib,
  stdenv,
  pkgs,
  darwin,
  cudaPackages,
}:
let
  inherit (stdenv) isDarwin;
  inherit (pkgs.config) cudaSupport;
  inherit (lib) makeBinPath optional optionals;
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

  buildInputs =
    optionals isDarwin [
      darwin.apple_sdk.frameworks.CoreGraphics
      darwin.apple_sdk.frameworks.Metal
      darwin.apple_sdk.frameworks.SystemConfiguration
    ]
    ++ optionals cudaSupport [ cudaPackages.cudatoolkit ];

  buildFeatures = [ ] ++ optional isDarwin "darwin" ++ optional cudaSupport "cuda";

  postInstall = ''
    wrapProgram $out/bin/host \
      --set PATH ${makeBinPath [ r0vm ]}
  '';
}
