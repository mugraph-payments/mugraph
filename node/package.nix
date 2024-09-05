{ mugraph, pkgs, ... }:
let
  inherit (mugraph.lib.defaults.rustPlatform) buildRustPackage;
  inherit (mugraph.lib.defaults) cargoLock;
in
buildRustPackage {
  name = "mugraph-node";
  src = ./..;

  cargoBuildFlags = "-p mugraph-node";

  nativeBuildInputs = with pkgs; [ protobuf ];

  inherit cargoLock;
}
