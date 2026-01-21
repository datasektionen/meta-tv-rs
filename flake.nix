{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      crane,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
        craneLib = crane.mkLib pkgs;
        src = craneLib.cleanCargoSource ./.;
        libs = [ pkgs.openssl ];
        commonArgs = {
          inherit src;
          strictDeps = true;

          buildInputs = libs;
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        crate = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
          }
        );
      in
      {
        packages = {
          default = crate;
        };

        devShell = craneLib.devShell {
          packages = with pkgs; [
            rustfmt
            rust-analyzer
            clippy
            sea-orm-cli
            pkg-config

            trunk
            leptosfmt
            lld
            tailwindcss_4
          ];

          env = {
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath libs;
            # Required by rust-analyzer
            RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          };
        };

        formatter = pkgs.nixfmt-tree;
      }
    )
    // {
      inherit self;
    };
}
