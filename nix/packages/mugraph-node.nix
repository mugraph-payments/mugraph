{ mugraph, pkgs, ... }:
let
  inherit (mugraph.lib.defaults.rustPlatform) buildRustPackage;
  inherit (mugraph.lib.defaults) root;
in
buildRustPackage {
  name = "mugraph-node";
  src = root;

  cargoBuildFlags = "-p mugraph-node";

  nativeBuildInputs = with pkgs; [ protobuf ];

  cargoLock = {
    lockFile = "${root}/Cargo.lock";

    outputHashes = {
      "redb-2.1.2" = "sha256-I4aDw0o0fYuU2ObDHZxSEG6tY1ad1IoyqhqAcfPMFzQ=";
    };
  };
}
