{ writeShellApplication }:
let
  inherit (builtins) readFile;
in
writeShellApplication {
  name = "rustup";
  text = readFile ./rustup-mock.sh;
}
