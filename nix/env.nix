{ pkgs }:
let
  inherit (pkgs) system;
  inherit (pkgs.lib) concatStringsSep;
  inherit (pkgs.mugraph.dependencies) rust;
  inherit (pkgs.mugraph.packages) r0vm;

  platform =
    {
      aarch64-darwin = "darwin";
      x86_64-darwin = "darwin";
      x86_64-linux = "linux";
      aarch64-linux = "linux";
    }
    .${system};

  useLinker = linker: [
    "-C"
    "linker=${pkgs.clang}"
    "-C"
    "link-arg=--ld-path=${linker}"
  ];

  globalRustflags = {

  };

  platformRustflags =
    globalRustflags
    // {
      darwin = useLinker pkgs.mold;
      linux = useLinker pkgs.lld;
    }
    ."${platform}";
in
{
  RUSTFLAGS = concatStringsSep " " platformRustflags.${platform};
  RUST_LOG = "info";
  RISC0_RUST_SRC = "${rust}/lib/rustlib/src/rust";
  RISC0_SERVER_PATH = "${r0vm}/bin/r0vm";
}
