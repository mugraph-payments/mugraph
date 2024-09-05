{ mugraph, pkgs, ... }:
let
  inherit (mugraph.lib.defaults.rustPlatform) buildRustPackage;
  inherit (mugraph.lib.defaults) cargoLock;
in
buildRustPackage {
  name = "mugraph-simulator";
  src = ./..;

  cargoBuildFlags = "-p mugraph-simulator";

  nativeBuildInputs = with pkgs; [ protobuf ];

  inherit cargoLock;
}
