{
  zlib,
  stdenv,
  fetchurl,
  mugraph,
  ...
}:
let
  inherit (stdenv) isDarwin mkDerivation;

  arch = if isDarwin then "aarch64-apple-darwin" else "x86_64-unknown-linux-gnu";
  tarballChecksum =
    if isDarwin then
      "sha256:0zx3lky6jh572gzvdfb43r3f4g62iwpj4i0zfv4vw343v8wk6jxv"
    else
      "sha256:1ll3vzmqiglplbrv8r7v0llnnpilpwd2c3b3ngph1yhykr39d12f";

  baseUrl = "https://github.com/risc0/rust/releases/download/r0.1.78.0";

  risc0-rust-tarball = fetchurl {
    url = "${baseUrl}/rust-toolchain-${arch}.tar.gz";
    sha256 = tarballChecksum;
  };
in
mkDerivation {
  name = "risc0-rust";

  inherit (mugraph.dependencies.rust) meta;

  unpackPhase = "true";
  nativeBuildInputs = [ zlib ];
  dontBuild = true;

  installPhase = ''
    mkdir -p $out
    cd $out
    tar xzf ${risc0-rust-tarball}

    rm -rf bin lib/*.dylib lib/rustlib/${arch}

    runHook postInstall
    runHook autoPatchelfHook
  '';
}
