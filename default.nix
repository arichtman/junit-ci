{
  pkgs ? import <nixpkgs> { } 
}:
pkgs.rustPlatform.buildRustPackage rec {
  pname = "junit-ci";
  version = "somever";
  src = pkgs.lib.cleanSource ./.;
  cargoLock.lockFile = ./Cargo.lock;
  #buildInputs = [];
  #buildPhase = "cargo build";
  #installPhase = "";
  meta = with pkgs.lib; {
    description = "junit-ci description";
    homepage = "https://github.com/arichtman/junit-ci";
    license = licenses.agpl3;
    platforms = platforms.unix;
    maintainers = ["Ariel"];
    mainProgram = "junitci";
  };
}