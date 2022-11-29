{ pkgs ? import <nixpkgs> {}}:

pkgs.mkShell {
  buildInputs = with pkgs; [
    rustup
    rustc
    cargo
    rustfmt
    rust-analyzer
    clippy
  ];
  shellHook = ''
  rustup override set nightly
  export PATH="$PATH:$HOME/.cargo/bin"
  '';
  RUST_BACKTRACE = 1;
}
