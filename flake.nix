{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

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
    rust-build = pkgs.rust-bin.stable.latest.default.override {
      extensions = [ "rust-src" ];
      targets = [ ];
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
      ];
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
    };
    packages.${system}.default = wikimark;
  };
}
