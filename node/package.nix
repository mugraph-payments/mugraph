{ mugraph, pkgs, ... }:
let
  inherit (mugraph.lib.defaults.rustPlatform) buildRustPackage;
  inherit (mugraph.lib.defaults) outputHashes root;
in
buildRustPackage {
  name = "mugraph-node";
  src = ./.;

  nativeBuildInputs = with pkgs; [ protobuf ];

  cargoLock = {
    inherit outputHashes;
    lockFile = "${root}/Cargo.lock";

  };
}
