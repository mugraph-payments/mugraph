{ fetchurl, stdenv, ... }:
let
  inherit (stdenv) isDarwin mkDerivation;

  arch = if isDarwin then "aarch64-apple-darwin" else "x86_64-unknown-linux-gnu";

  tarballChecksum =
    if isDarwin then
      "sha256:0zx3lky6jh572gzvdfb43r3f4g62iwpj4i0zfv4vw343v8wk6jxv"
    else
      "sha256-+IXVTmBH0MEI0x1rTfVfPgWUb04KaZzKK9vW1603dK8=";

  baseUrl = "https://github.com/risc0/rust/releases/download/r0.1.78.0";

  source = fetchurl {
    url = "${baseUrl}/rust-toolchain-${arch}.tar.gz";
    sha256 = tarballChecksum;
  };
in
mkDerivation {
  name = "risc0-toolchain";
  unpackPhase = "true";
  dontBuild = true;

  installPhase = ''
    mkdir -p $out/tmp
    mkdir -p $out/lib/rustlib
    tar xzf ${source} -C $out/tmp
    mv $out/tmp/lib/rustlib/riscv32im-risc0-zkvm-elf $out/lib/rustlib
    rm -rf $out/tmp
    runHook postInstall
  '';
}
