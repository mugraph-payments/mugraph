inputs: final: prev:
let
  inherit (inputs) rust-dev-tools;

  rdt = rust-dev-tools.setup prev {
    name = "mu";
    rust = rust-dev-tools.version.fromToolchainFile ./../rust-toolchain.toml;
  };
in
{
  mugraph = {
    inherit rdt;

    r0vm = final.callPackage ./r0vm.nix {
      inherit rdt;

      risc0-source = inputs.risc0;
    };
    buildRisc0Package = final.callPackage ./risczero.nix { toolchain = rdt.rust; };
  };
}
