{ mugraph, ... }:
let
  inherit (mugraph.lib.defaults.rustPlatform) buildRustPackage;
  inherit (mugraph.lib.defaults) root;
in
buildRustPackage {
  name = "mugraph-node";
  src = root;

  cargoBuildFlags = "-p mugraph-node";
  cargoLock.lockFile = "${root}/Cargo.lock";
}
