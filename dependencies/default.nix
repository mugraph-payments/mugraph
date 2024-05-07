{ pkgs, inputs }:
{
  leo = pkgs.rustPlatform.buildRustPackage {
    name = "leo";
    src = pkgs.lib.cleanSourceWith { src = inputs.leo; };
    cargoSha256 = "sha256-SgqsfefLU7l/OPgi3xdYOwKfiyq/zLbfghJ1qw8I+9U=";
    buildInputs = with pkgs; [ openssl ];
    nativeBuildInputs = with pkgs; [
      cmake
      pkg-config
    ];
  };
}
