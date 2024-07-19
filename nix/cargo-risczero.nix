{
  rdt,
  risc0-source,
  openssl,
}:
rdt.buildRustPackage {
  pname = "cargo-risczero";
  version = "1.0.3";
  src = risc0-source;

  buildAndTestSubdir = "risc0/cargo-risczero";
  buildInputs = [ openssl.dev ];

  doCheck = false;
  cargoLock.lockFile = "${risc0-source}/Cargo.lock";

  meta = {
    homepage = "https://github.com/risc0/risc0";
    description = "cargo-risczero";
  };
}
