{
  description = "Development environment for System Initiative";

  # Flake inputs
  inputs = {
    # rust-overlay is designed to work with nixos-unstable
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    # TODO(nick): re-enable once remote caching is enabled.
    # buck2 = {
    #   url = "path:nix/buck2";
    #   inputs.nixpkgs.follows = "nixpkgs";
    # };
    # reindeer = {
    #   url = "path:nix/reindeer";
    #   inputs.nixpkgs.follows = "nixpkgs";
    # };
  };

  # Flake outputs
  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
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

        # TODO(nick): re-enable once remote caching is enabled.
        # buck2-pkg = buck2.packages.${system}.buck2;
        # reindeer-pkg = reindeer.packages.${system}.reindeer;
      in with pkgs; {
        devShells.default = mkShell {
          buildInputs = [
            # buck2-pkg
            # reindeer-pkg
            automake
            bash
            clang
            coreutils
            docker-compose
            gcc
            git
            gnumake
            jq
            libtool
            lld
            nodejs-18_x
            nodePackages.pnpm
            nodePackages.typescript
            nodePackages.typescript-language-server
            openssl
            pgcli
            pkg-config
            postgresql_14
            protobuf
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
          depsTargetTarget = [ awscli butane kubeval nodejs-18_x skopeo ];
          # This is awful, but necessary (until we find a better way) to
          # be able to `cargo run` anything that compiles against
          # openssl. Without this, ld is unable to find libssl.so.3 and
          # libcrypto.so.3.
          #
          # If we were packaging this up as a flake, instead of only
          # using nix for the development environment, we'd be using
          # wrapProgram with something like
          # `--prefix LD_LIBRARY_PATH : ${lib.makeLibraryPath [ openssl ]}`
          # to make sure the things we're compiling are always using the
          # version of openssl they were compiled against.
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [ pkgs.openssl ];
        };
      });
}
