{
  description = "singh3 discord bot";

  inputs = {
    nixpkgs.url = github:nixos/nixpkgs/nixos-unstable;
    utils.url = github:numtide/flake-utils;
    rust-overlay.url = github:oxalica/rust-overlay;
    cargo2nix.url = github:cargo2nix/cargo2nix;
  };

  outputs = { self, nixpkgs, utils, rust-overlay, cargo2nix }:
    utils.lib.eachDefaultSystem
      (system:
        let

          overlays =
            [
              (import "${cargo2nix}/overlay")
              rust-overlay.overlay
            ];

          pkgs = import nixpkgs {
            inherit system overlays;
          };

          rustPkgs = pkgs.rustBuilder.makePackageSet' {
            rustChannel = "latest";
            packageFun = import ./Cargo.nix;
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

          packages = {
            default = (rustPkgs.workspace.singh3 { }).bin;
          };

          defaultPackage = packages.default;
        }
      );
}
