{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { naersk, nixpkgs, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
      naersk-lib = pkgs.callPackage naersk { };
    in {
      packages.${system}.default = naersk-lib.buildPackage { src = ./.; };
      devShells.${system}.default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [ rustc cargo pkg-config ];
        RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
        PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
      };
    };
}
