{ mugraph, ... }:
let
  inherit (mugraph.lib.defaults.rustPlatform) buildRustPackage;
  inherit (mugraph.lib.defaults) root;
in
buildRustPackage {
  name = "mugraph-simulator";
  src = root;

  cargoBuildFlags = "-p mugraph-simulator";
  cargoLock.lockFile = "${root}/Cargo.lock";
}
