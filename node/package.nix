{ mugraph, pkgs, ... }:
let
  inherit (mugraph.lib.rustPlatform) buildRustPackage;
  inherit (mugraph.lib) cargoLock;
in
buildRustPackage {
  name = "mugraph-node";
  src = ./..;

  cargoBuildFlags = "-p mugraph-node";

  nativeBuildInputs = with pkgs; [ protobuf ];

  inherit cargoLock;
}
