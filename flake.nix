{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [
          (import rust-overlay)
        ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rust-version = "latest";
        rust-toolchain = pkgs.rust-bin.stable.${rust-version}.default.override
          {
            extensions = [
              "rust-src"
              "rust-analyzer"
            ];
          };
      in
      {
        devShell = with pkgs; mkShell {
          buildInputs = [
            openssl
            pkg-config
            rust-toolchain
          ];
        };
        formatter = pkgs.nixpkgs-fmt;
      }
    );
}
  
