inputs:
{ pkgs }:
let
  inherit (builtins)
    attrNames
    filter
    readDir
    baseNameOf
    ;

  inherit (pkgs) system;
  inherit (pkgs.lib)
    concatStringsSep
    hasSuffix
    listToAttrs
    removeSuffix
    ;

  platform =
    {
      aarch64-darwin = "darwin";
      x86_64-darwin = "darwin";
      x86_64-linux = "linux";
      aarch64-linux = "linux";
    }
    .${system};

  useLinker =
    linker:
    concatStringsSep " " [
      "-C"
      "linker=${pkgs.clang}/bin/clang"
      "-C"
      "link-arg=--ld-path=${linker}"
    ];

  RUSTFLAGS =
    {
      darwin = useLinker "${pkgs.lld}/bin/ld64.lld";
      linux = useLinker "${pkgs.mold}/bin/mold";
    }
    ."${platform}";

  root = ./..;

  rust = pkgs.rust-bin.fromRustupToolchainFile "${root}/rust-toolchain.toml";
in
{
  inherit inputs;

  defaults = {
    inherit rust root;

    rustPlatform = pkgs.makeRustPlatform {
      rustc = rust;
      cargo = rust;
    };

    env = {
      inherit RUSTFLAGS;

      RUST_LOG = "info";
    };

    cargoLock = {
      lockFile = ../Cargo.lock;

      outputHashes = {
        "redb-2.1.2" = "sha256-I4aDw0o0fYuU2ObDHZxSEG6tY1ad1IoyqhqAcfPMFzQ=";
      };
    };
  };

  buildPackageSet =
    dir:
    let
      files = filter (hasSuffix ".nix") (attrNames (readDir dir));

      toAttr = n: {
        name = removeSuffix ".nix" n;
        value = pkgs.callPackage "${dir}/${n}" { };
      };
    in
    listToAttrs (map toAttr files);
}
