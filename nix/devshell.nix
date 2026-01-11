{ lib
, mkShell
, git
, cargo-edit
, cargo-watch
, rust-analyzer-unwrapped
, wikimark
}:

mkShell {
  packages = [
    git
    cargo-edit
    cargo-watch
    rust-analyzer-unwrapped
  ];
  inputsFrom = [
    wikimark
  ];
  RUST_SRC_PATH = "${wikimark.rust-build}/lib/rustlib/src/rust/library";
  inherit (wikimark) CARGO_BUILD_TARGET CARGO_BUILD_RUSTFLAGS;
}
