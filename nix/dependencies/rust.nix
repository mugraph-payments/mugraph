{
  rust-bin,
  symlinkJoin,
  pkgs,
  mugraph,
  ...
}:
let
  inherit (mugraph.dependencies) rustup-mock risc0-toolchain;

  rust = rust-bin.fromRustupToolchainFile ../../rust-toolchain.toml;
  env = import ../env.nix { inherit pkgs; };
in
symlinkJoin {
  inherit (rust) meta;

  name = "mugraph-rust";

  paths = [
    rust
    risc0-toolchain
    rustup-mock
  ];

  postFixup = ''
    wrapProgram $out/bin/rustc \
      --set RISC0_RUST_SRC ${rust}/lib/rustlib/src/rust \
      --set RUSTFLAGS ${env.RUSTFLAGS}

    wrapProgram $out/bin/cargo \
      --set RISC0_RUST_SRC ${rust}/lib/rustlib/src/rust \
      --set RUSTFLAGS ${env.RUSTFLAGS}
  '';
}
