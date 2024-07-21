inputs: final: prev:
let
  inherit (builtins)
    attrNames
    attrValues
    elemAt
    listToAttrs
    match
    readDir
    ;

  inherit (prev) mkShell;
  inherit (prev.lib) concatStringsSep;

  dependencies = listToAttrs (
    map (file: {
      name = elemAt (match "(.*)\\.nix" file) 0;
      value = final.callPackage (./dependencies + "/${file}") { };
    }) (attrNames (readDir ./dependencies))
  );

  rust = final.symlinkJoin {
    inherit (final.rust-bin.nightly.latest.complete) meta;

    name = "mugraph-rustc";
    paths = [
      (final.rust-bin.nightly.latest.complete)
      dependencies.risc0-rust
    ];
  };

  rustPlatform = final.makeRustPlatform {
    rustc = rust;
    cargo = rust;
  };

  packages = listToAttrs (
    map (file: {
      name = elemAt (match "(.*)\\.nix" file) 0;
      value = final.callPackage (./packages + "/${file}") { };
    }) (attrNames (readDir ./packages))
  );

  systemFlags = {
    "x86_64-darwin" = [ "-C link-arg=-fuse-ld=lld" ];
    "aarch64-darwin" = systemFlags."x86_64-darwin";
    "x86_64-linux" = [
      "-C link-arg=-fuse-ld=mold"
      "-C link-arg=-Wl,--separate-debug-file"
    ];
    "aarch64-linux" = systemFlags."x86_64-linux";
  };

  systemDeps = {
    "aarch64-darwin" = with prev; [
      darwin.apple_sdk.frameworks.SystemConfiguration
      lld
    ];
    "x86_64-darwin" = systemDeps."aarch64-darwin";
    "x86_64-linux" = with prev; [ mold ];
    "aarch64-linux" = systemDeps."x86_64-linux";
  };

  devShells.default = mkShell {
    name = "mu-shell";

    packages = [
      rust
      (attrValues dependencies)
      systemDeps.${final.system}

      final.cargo-nextest
      final.cargo-watch
    ];

    RISC0_DEV_MODE = 1;
    RISC0_RUST_SRC = "${rust}/lib/rustlib/src/rust";
    RUSTFLAGS = concatStringsSep " " systemFlags.${final.system};
  };
in
{
  mugraph = {
    inherit devShells inputs packages;

    dependencies = dependencies // {
      inherit rust rustPlatform;
    };
  };
}
