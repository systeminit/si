{
  description = "Development environment for System Initiative";

  # Flake inputs
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  # Flake outputs
  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [
          (self: super: {
            nodejs = super.nodejs-18_x;
            pnpm = super.nodePackages.pnpm;
          })
          (import rust-overlay)
        ];

        pkgs = import nixpkgs { inherit overlays system; };

        rustVersion = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain;
        rust-toolchain = rustVersion.override {
          extensions = [ "rust-analyzer" "rust-src" ];
        };

      in with pkgs; {
        devShells.default = mkShell {
          packages = [
            bash
            buck2
            clang
            docker-compose
            gh
            jq
            lld
            nodejs
            pgcli
            pnpm
            protobuf
            python3
            reindeer
            rust-toolchain
            tilt
          ] ++ lib.optionals pkgs.stdenv.isDarwin [
            libiconv
            darwin.apple_sdk.frameworks.Security
            darwin.apple_sdk.frameworks.SystemConfiguration
          ];
          depsTargetTarget = [
            awscli
            butane
            kubeval
            skopeo
          ];
        };
      });
}
