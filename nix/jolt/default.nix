{
  mugraph,
  lib,
  stdenv,
  darwin,
  rustup,
  ...
}:
let
  inherit (lib) optionals;
  inherit (mugraph.lib.defaults.rustPlatform) buildRustPackage;
  inherit (mugraph.lib.inputs) jolt;
  inherit (stdenv) isDarwin;
  inherit (darwin.apple_sdk.frameworks) IOKit SystemConfiguration;
in
buildRustPackage {
  name = "jolt";
  src = jolt;

  cargoBuildFlags = "--bin jolt";
  nativeBuildInputs = optionals isDarwin [
    IOKit
    SystemConfiguration
  ];
  propagatedBuildInputs = [
    rustup
  ];

  postPatch = ''
    ln -s ${./Cargo.lock} Cargo.lock
  '';

  cargoLock = {
    lockFile = ./Cargo.lock;

    outputHashes = {
      "ark-ec-0.4.2" = "sha256-E/+yVQwsUTux7KpC5jnG4E3CKPzyhHV6LmdECVCDe1g=";
      "binius_field-0.1.0" = "sha256-YKyW68yBPakl1v8ioyhgNQcV6DMTLfK5FgI7AWISpVc=";
      "p3-util-0.1.0" = "sha256-UYHz6GyNn3VkIEm4HDCsP2sLwAKfBi01IiV5X1AE1aM=";
    };
  };
}
