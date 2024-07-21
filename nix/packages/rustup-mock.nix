{ writeShellApplication, mugraph, ... }:
writeShellApplication {
  name = "rustup";
  text = ''
    if [[ "$1" = "toolchain" ]]
    then
      printf "risc0\n"
    elif [[ "$1" = "+risc0" ]]
    then
      printf "${mugraph.rust}/bin/rustc"
    fi
  '';
}
