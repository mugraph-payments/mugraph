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

  snarkvm = pkgs.rustPlatform.buildRustPackage {
    name = "snarkvm";
    src = pkgs.lib.cleanSourceWith { src = inputs.snarkvm; };
    cargoSha256 = "sha256-xyZeonAs3AWFgt4Jl+siaER4ERJrTmnJOtqjc2IG5Og=";
    buildInputs = with pkgs; [ openssl ];
    nativeBuildInputs = with pkgs; [ pkg-config ];
    doCheck = false;
  };
}
