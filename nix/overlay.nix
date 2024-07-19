inputs: final: prev:
let
  inherit (inputs) rust-dev-tools;

  rdt = rust-dev-tools.setup prev {
    name = "mugraph";
    rust = rust-dev-tools.version.nightly;
    dependencies = [ ];
  };
in
{
  mugraph = {
    inherit rdt;

    r0vm = final.callPackage ./r0vm.nix {
      inherit rdt;
      risc0Source = inputs.risc0;
    };
    buildRisc0Package = final.callPackage ./lib/buildRisc0Package.nix { };
  };
}
