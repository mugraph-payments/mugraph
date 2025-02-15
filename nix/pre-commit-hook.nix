{
  mugraph,
  system,
}:
let
  inherit (mugraph.inputs) pre-commit-hooks;
  inherit (mugraph.lib) rust root;
in
pre-commit-hooks.lib.${system}.run {
  src = root;

  hooks = {
    nixfmt-rfc-style.enable = true;

    rustfmt = {
      enable = true;

      packageOverrides = {
        cargo = rust;
        rustfmt = rust;
      };
    };
  };
}
