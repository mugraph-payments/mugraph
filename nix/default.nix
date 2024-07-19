inputs: final: prev:
let
  inherit (inputs) rust-dev-tools;

  inherit (builtins) fetchurl;
  inherit (prev.stdenv) isDarwin mkDerivation;
  inherit (prev) symlinkJoin writeShellApplication mkShell;

  rustup-mock = writeShellApplication {
    name = "rustup";
    text = ''
      if [[ "$1" = "toolchain" ]]
      then
        printf "risc0\n"
      elif [[ "$1" = "+risc0" ]]
      then
        printf "${rdt.rust}/bin/rustc"
      fi
    '';
  };

  tarballFile =
    if isDarwin then
      "rust-toolchain-aarch64-apple-darwin.tar.gz"
    else
      "rust-toolchain-x86_64-unknown-linux-gnu.tar.gz";
  tarballChecksum =
    if isDarwin then
      "sha256:0zx3lky6jh572gzvdfb43r3f4g62iwpj4i0zfv4vw343v8wk6jxv"
    else
      "sha256:1ll3vzmqiglplbrv8r7v0llnnpilpwd2c3b3ngph1yhykr39d12f";

  baseUrl = "https://github.com/risc0/rust/releases/download/r0.1.78.0";

  risc0-rust-tarball = fetchurl {
    url = "${baseUrl}/${tarballFile}";
    sha256 = tarballChecksum;
  };

  risc0-rust = mkDerivation {
    name = "risc0-rust";

    unpackPhase = "true";
    nativeBuildInputs = [ prev.zlib ];
    dontBuild = true;

    installPhase = ''
      mkdir -p $out
      cd $out
      tar xzf ${risc0-rust-tarball}
      chmod +x bin/*
      runHook postInstall
      runHook autoPatchelfHook
    '';
  };

  rdt = rust-dev-tools.setup prev {
    name = "mu";
    rust = rust-dev-tools.version.fromToolchainFile ../rust-toolchain.toml;
  };

  r0vm = final.callPackage ./r0vm.nix {
    inherit rdt;
    inherit risc0-rust;
    risc0-source = inputs.risc0;
  };

  cargo-risczero = final.callPackage ./cargo-risczero.nix {
    inherit rdt;
    risc0-source = inputs.risc0;
  };

  buildRisc0Package = final.callPackage ./buildRisc0Package.nix { inherit risc0-rust; };

  devShell = mkShell {
    name = "mu-shell";
    inputsFrom = [ rdt.devShell ];
    packages = [
      r0vm
      rustup-mock
      cargo-risczero
    ];

    RISC0_RUST_SRC = "${risc0-rust}/lib/rustlib/src/rust";
    RISC0_DEV_MODE = 1;
  };
in
{
  mugraph = {
    inherit
      buildRisc0Package
      devShell
      r0vm
      rdt
      rustup-mock
      risc0-rust
      ;
  };
}
