{
  openssl,
  darwin,
  mugraph,
}:
mugraph.dependencies.rustPlatform.buildRustPackage {
  pname = "cargo-risczero";
  version = "1.0.3";
  src = mugraph.inputs.risc0;

  buildAndTestSubdir = "risc0/cargo-risczero";
  buildInputs = [
    openssl.dev
    darwin.apple_sdk.frameworks.SystemConfiguration
  ];

  doCheck = false;
  cargoLock.lockFile = "${mugraph.inputs.risc0}/Cargo.lock";

  meta = {
    homepage = "https://github.com/risc0/risc0";
    description = "cargo-risczero";
  };
}
