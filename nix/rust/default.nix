{
  callPackage,
  lib,
  lld,
  mold,
  rust-bin,
  stdenv,
  symlinkJoin,
  writeShellApplication,
  ...
}:
let
  inherit (lib) concatStringsSep;
  inherit (stdenv) isDarwin;
  inherit (builtins) readFile;

  platform = if isDarwin then "darwin" else "linux";

  systemFlags = {
    darwin = [ "-C link-arg=-fuse-ld=lld" ];
    linux = [ "-C link-arg=-fuse-ld=mold" ];
  };

  systemDeps = {
    darwin = [ lld ];
    linux = [ mold ];
  };

  rust = rust-bin.fromRustupToolchainFile ../../rust-toolchain.toml;

  risc0-toolchain = callPackage ./risc0.nix { };
  RUSTFLAGS = concatStringsSep " " systemFlags.${platform};

  rustup-mock = writeShellApplication {
    name = "rustup";
    text = readFile ./rustup-mock.sh;
  };
in
symlinkJoin {
  inherit (rust) meta;

  name = "mugraph-rust";

  paths = [
    risc0-toolchain
    rust
    rustup-mock
  ] ++ systemDeps.${platform};

  postFixup = ''
    wrapProgram $out/bin/rustc \
      --set RISC0_RUST_SRC ${rust}/lib/rustlib/src/rust \
      --set RUSTFLAGS ${RUSTFLAGS}

    wrapProgram $out/bin/cargo \
      --set RISC0_RUST_SRC ${rust}/lib/rustlib/src/rust \
      --set RUSTFLAGS ${RUSTFLAGS}
  '';

  passthru = {
    inherit RUSTFLAGS;
  };
}
