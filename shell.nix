{
  pkgs ? import <nixpkgs> { },
}:
let
  libs = with pkgs; [
    openssl
  ];
  libPath = pkgs.lib.makeLibraryPath libs;
in
pkgs.mkShell {
  packages =
    with pkgs;
    [
      cargo
      rustc
      rustfmt
      rust-analyzer
      clippy
      sea-orm-cli
      pkg-config

      trunk
      leptosfmt
      lld
      tailwindcss_4
    ]
    ++ libs;
  env = {
    LD_LIBRARY_PATH = libPath;
    # Required by rust-analyzer.
    RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
  };
}
