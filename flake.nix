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
        rec {
          devShells = with pkgs; {
            default = mkShell
              {
                buildInputs = [
                  rust-bin.nightly.latest.default
                  rust-analyzer
                  postgresql
                ];
              };
            withDB = mkShell
              {
                buildInputs = [
                  rust-bin.nightly.latest.default
                  postgresql
                ];
              };
            bare = mkShell
              {
                buildInputs = [
                  rust-bin.nightly.latest.default
                ];
              };
            withLSP = mkShell
              {
                buildInputs = [
                  rust-bin.nightly.latest.default
                  rust-analyzer
                ];
              };
          };
          devShell = devShells.default;
          defaultPackage = pkgs.rustPlatform.buildRustPackage rec {
            pname = "singh3";
            version = "0.1.0";
            src = ./.;
            nativeBuildInputs = with pkgs; [
              rust-bin.nightly.latest.default
            ];
            cargoSha256 = "sha256-04yTexSkFpa3KQKVvfi7NM1j4V7m08kHDqw98bxXT5M=";
          };
        }
      );
}
