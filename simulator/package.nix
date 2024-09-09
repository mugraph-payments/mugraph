{ mugraph, pkgs, ... }:
let
  inherit (mugraph.lib.rustPlatform) buildRustPackage;
  inherit (mugraph.lib) cargoLock;
in
buildRustPackage {
  name = "mugraph-simulator";
  src = ./..;

  cargoBuildFlags = "-p mugraph-simulator";

  nativeBuildInputs = with pkgs; [ protobuf ];

  inherit cargoLock;
}
