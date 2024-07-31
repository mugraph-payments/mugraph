{
  mugraph,
  lib,
  stdenv,
  darwin,
}:
let
  inherit (stdenv) isDarwin;
  inherit (lib) optionals;
  inherit (mugraph.lib.defaults) env paths rustPlatform;
in
rustPlatform.buildRustPackage {
  pname = "mugraph-node";
  version = "0.0.1";
  src = paths.root;

  inherit env;

  doCheck = false;
  auditable = false;

  cargoLock = {
    lockFile = paths.cargoLock;

    outputHashes = {
      "crypto-bigint-0.5.5" = "sha256-7kCaAgyJKOD5C7Av0po+NMqpNgRoA478URwOK7VF7Mc=";
      "curve25519-dalek-4.1.2" = "sha256-tm3PFj/Y3JaJAcfv4jYexaA1wQbYy8NZjHiP6kGIlso=";
      "sha2-0.10.8" = "sha256-vuFQFlbDXEW+n9+Nx2VeWanggCSd6NZ+GVEDFS9qZ2M=";
    };
  };

  buildInputs = optionals isDarwin [ darwin.apple_sdk.frameworks.SystemConfiguration ];
}
