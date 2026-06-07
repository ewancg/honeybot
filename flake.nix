{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs =
    { nixpkgs
    , flake-utils
    , rust-overlay
    , ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        stdenv = pkgs.stdenvAdapters.useWildLinker pkgs.gccStdenv;
        buildInputs = [
          (pkgs.rust-bin.selectLatestNightlyWith
            (toolchain: toolchain.default.override {
              extensions = [ "rust-src" "rust-analyzer" "clippy" ];
            }))
        ];
        nativeBuildInputs = [
          pkgs.rustc
        ];
      in
      {
        allowUnfree = true;
        devShell = pkgs.mkShell rec {
          inherit stdenv buildInputs nativeBuildInputs;
          packages = buildInputs ++ nativeBuildInputs;

          LLVM_CONFIG_PATH = "${pkgs.llvm}/bin/llvm-config";
          RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
        };
      }
    );
}
