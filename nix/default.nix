inputs: final: prev:
let
  inherit (builtins)
    attrNames
    elemAt
    listToAttrs
    match
    readDir
    ;

  inherit (prev) mkShell;
  inherit (prev.lib) concatStringsSep;

  rust = final.rust-bin.stable.latest.complete;

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
    "x86_64-linux" = [ "-C link-arg=-fuse-ld=mold" ];
    "aarch64-linux" = systemFlags."x86_64-linux";
  };

  systemDeps = {
    "x86_64-darwin" = [
      rust
      final.lld
    ];
    "aarch64-darwin" = systemDeps."x86_64-darwin";
    "x86_64-linux" = [
      rust
      final.mold
    ];
    "aarch64-linux" = systemFlags."x86_64-linux";
  };

  checks.pre-commit = inputs.pre-commit-hooks.lib.${final.system}.run {
    src = ../.;
    hooks = {
      nixfmt = {
        enable = true;
        package = final.nixfmt-rfc-style;
      };

      rustfmt = {
        enable = true;
        packageOverrides = {
          cargo = rust;
          rustfmt = rust;
        };
      };
    };
  };

  devShells.default = mkShell {
    inherit (checks.pre-commit) shellHook;

    name = "mu-shell";

    packages = [
      checks.pre-commit.enabledPackages
      final.cargo-nextest
      final.cargo-watch
      systemDeps.${final.system}
    ];

    RUSTFLAGS = concatStringsSep " " systemFlags.${final.system};
  };
in
{
  mugraph = {
    inherit
      devShells
      inputs
      checks
      rust
      rustPlatform
      ;

    packages = packages // {
      default = packages.mugraph-node;
    };
  };
}
