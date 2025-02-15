{ mugraph, system }:
let
  inherit (mugraph.inputs) pre-commit-hooks;
  inherit (mugraph.lib) root;
in
pre-commit-hooks.lib.${system}.run {
  src = root;

  hooks.treefmt = {
    enable = true;
    package = mugraph.formatter;
  };
}
