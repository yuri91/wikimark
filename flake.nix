{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";

    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";

    naersk.url = "github:nmattia/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, rust-overlay, naersk, ... } @ inputs:
  let
    system = "x86_64-linux";
    pkgs = import nixpkgs {
      inherit system;
      overlays = [ rust-overlay.overlays.default ];
    };
    CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
    CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
    rust-build = pkgs.rust-bin.nightly.latest.default.override {
      extensions = [ "rust-src" ];
      targets = [ CARGO_BUILD_TARGET ];
    };
    naersk-lib = naersk.lib.${system}.override {
      rustc = rust-build;
      cargo = rust-build;
    };
    wikimark = naersk-lib.buildPackage {
      pname = "wikimark";
      root = ./.;
      buildInputs = with pkgs; [
      ];
      nativeBuildInputs = with pkgs; [
        rust-build
        pkgsStatic.stdenv.cc
      ];
      release = false;
      cargo_release = "--profile dist";
      inherit CARGO_BUILD_TARGET;
      inherit CARGO_BUILD_RUSTFLAGS;
    };
  in
  {
    devShell.${system} = pkgs.mkShell {
      packages = with pkgs; [
        git
        cargo-edit
        cargo-watch
        rust-analyzer-unwrapped
      ];
      inputsFrom = with pkgs; [
        wikimark
      ];
      RUST_SRC_PATH = "${rust-build}/lib/rustlib/src/rust/library";
      inherit CARGO_BUILD_TARGET;
      inherit CARGO_BUILD_RUSTFLAGS;
    };
    packages.${system} = {
      default = wikimark;
    };
  };
}
