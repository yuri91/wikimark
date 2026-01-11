{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";

    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";

    naersk.url = "github:nmattia/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, rust-overlay, naersk, ... }:
  let
    system = "x86_64-linux";
    lib = nixpkgs.lib;
    pkgs = import nixpkgs {
      inherit system;
      overlays = [ rust-overlay.overlays.default ];
    };
    wikimark = pkgs.callPackage ./nix/package.nix {
      naersk = naersk.lib.${system};
    };
  in
  {
    devShells.${system}.default = pkgs.callPackage ./nix/devshell.nix {
      inherit wikimark;
    };

    packages.${system}.default = wikimark;

    nixosModules.default = { pkgs, ... }: {
      imports = [ ./nix/nixos-module.nix ];
      services.wikimark.package = lib.mkDefault self.packages.${pkgs.system}.default;
    };
  };
}
