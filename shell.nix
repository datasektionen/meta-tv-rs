{pkgs ? import <nixpkgs> {}}:
pkgs.mkShell {
  buildInputs = with pkgs; [
    cargo
    rustc
    rustfmt
    rust-analyzer
    clippy
    sea-orm-cli

    trunk
    leptosfmt
    lld
    tailwindcss_4
  ];
}
