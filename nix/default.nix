inputs: final: prev:
let
  inherit (builtins)
    attrNames
    attrValues
    listToAttrs
    readDir
    ;

  inherit (prev) mkShell;
  inherit (prev.lib) removeSuffix;

  env = import ./env.nix { pkgs = final; };

  buildPackageSet =
    dir:
    let
      dirContents = readDir dir;
      entries = map (n: {
        name = removeSuffix ".nix" n;
        path = n;
      }) (attrNames dirContents);
      build =
        { name, path }:
        {
          inherit name;
          value = final.callPackage path { };
        };
    in
    listToAttrs (map build entries);

  packages = buildPackageSet ./packages;
  dependencies = buildPackageSet ./dependencies;

  rustPlatform = final.makeRustPlatform {
    rustc = dependencies.rust;
    cargo = dependencies.rust;
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
          cargo = dependencies.rust;
          rustfmt = dependencies.rust;
        };
      };
    };
  };

  devShells.default =
    mkShell {
      inherit (checks.pre-commit) shellHook;

      name = "mu-shell";

      packages = [
        checks.pre-commit.enabledPackages
        (attrValues dependencies)
      ];

      RISC0_EXECUTOR = "ipc";
    }
    // env;
in
{
  mugraph = {
    inherit
      checks

      devShells
      inputs
      packages
      ;

    dependencies = dependencies // {
      inherit rustPlatform;
    };
  };
}
