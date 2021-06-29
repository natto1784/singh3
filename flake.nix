{
  description = "A simple filehost written in rust";

  inputs = {
    nixpkgs.url = github:nixos/nixpkgs/nixos-unstable;
    utils.url = github:numtide/flake-utils;
    rust-overlay.url = github:oxalica/rust-overlay;
  };

  outputs = { self, nixpkgs, utils, rust-overlay }: 
  utils.lib.eachDefaultSystem
  (system:
    let 
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };
    in
    {
      devShell = with pkgs; mkShell {
        buildInputs = [
          rust-bin.nightly.latest.default
          rust-analyzer
        ];
      };
      defaultPackage = pkgs.rustPlatform.buildRustPackage rec {
        pname = "singh3";
        version = "0.1.0";
        src = ./. ;
        nativeBuildInputs = with pkgs; [
          rust-bin.nightly.latest.default
        ];
        cargoSha256 = "sha256-K+WHOEo6reNfcs7pOZZmHZfZl4pUqlykfTdqgSyVURU=";
      };
    }
  );
}
