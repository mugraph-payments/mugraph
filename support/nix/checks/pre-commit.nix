{
  mugraph,
  nixfmt-rfc-style,
  system,
}:
let
  inherit (mugraph.inputs) pre-commit-hooks;
  inherit (mugraph.lib.defaults) rust;
in
pre-commit-hooks.lib.${system}.run {
  src = ../../.;

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
