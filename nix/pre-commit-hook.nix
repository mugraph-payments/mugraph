{
  mugraph,
  nixfmt-rfc-style,
  system,
}:
let
  inherit (mugraph.inputs) pre-commit-hooks;
  inherit (mugraph.lib) rust root;
in
pre-commit-hooks.lib.${system}.run {
  src = root;

  hooks = {
    nixfmt = {
      enable = true;
      package = nixfmt-rfc-style;
    };

    rustfmt = {
      enable = true;
      packageOverrides = {
        cargo = rust;
        rustfmt = rust;
      };
    };
  };
}
