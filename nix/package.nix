{ lib
, pkgsStatic
, rust-bin
, naersk
}:

let
  CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
  CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
  rust-build = rust-bin.stable.latest.default.override {
    extensions = [ "rust-src" ];
    targets = [ CARGO_BUILD_TARGET ];
  };
  naersk-lib = naersk.override {
    rustc = rust-build;
    cargo = rust-build;
  };
in
naersk-lib.buildPackage {
  pname = "wikimark";
  root = ./..;
  nativeBuildInputs = [
    rust-build
    pkgsStatic.stdenv.cc
  ];
  release = false;
  cargo_release = "--profile dist";
  inherit CARGO_BUILD_TARGET CARGO_BUILD_RUSTFLAGS;

  passthru = {
    inherit CARGO_BUILD_TARGET CARGO_BUILD_RUSTFLAGS rust-build;
  };
}
