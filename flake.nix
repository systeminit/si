{
  description = "Development environment for System Initiative";

  # Flake inputs
  inputs = {
    # rust-overlay is designed to work with nixos-unstable
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    buck2-subflake = {
      url = "path:nix/buck2";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  # Flake outputs
  outputs = { self, nixpkgs, flake-utils, rust-overlay, buck2-subflake, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [
          (import rust-overlay)

          (self: super: {
            rustToolchain =
              super.rust-bin.fromRustupToolchainFile ./rust-toolchain;
          })
        ];
        pkgs = import nixpkgs { inherit system overlays; };

        # Context: https://github.com/NixOS/nixpkgs/issues/226677
        buck2 = buck2-subflake.packages.${system}.buck2;

        # Ensure pnpm uses our defined node toolchain and does not download its own.
        pinnedNode = pkgs.nodejs-18_x;
        nodePackagesWithPinnedNode =
          pkgs.nodePackages.override { nodejs = pinnedNode; };

      in with pkgs; {
        devShells.default = mkShell {
          buildInputs = [
            automake
            bash
            buck2
            clang
            coreutils
            docker-compose
            gcc
            gnumake
            jq
            libtool
            lld
            nodePackagesWithPinnedNode.pnpm
            nodePackagesWithPinnedNode.typescript
            nodePackagesWithPinnedNode.typescript-language-server
            pgcli
            pinnedNode
            pkg-config
            postgresql_14
            protobuf
            reindeer

            (rustToolchain.override {
              # This really should be augmenting the extensions, instead of
              # completely overriding them, but since we're not setting up
              # any extensions in our rust-toolchain file, it should be
              # fine for now.
              extensions = [ "rust-src" "rust-analyzer" ];
            })
          ] ++ lib.optionals pkgs.stdenv.isDarwin [
            libiconv
            darwin.apple_sdk.frameworks.Security
          ];
          depsTargetTarget = [
            awscli
            butane
            kubeval
            skopeo

            # NOTE(nick): we may not need this if we are purely using pnpm's toolchain. More
            # investigation with veritech on NixOS is recommended.
            pinnedNode
          ];
        };
      });
}
