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

        # Pending an upstream Buck2 package we download pre-built system binaries.
        # References: https://github.com/NixOS/nixpkgs/issues/226677
        buck2 = pkgs.stdenv.mkDerivation rec {
          # These are the primary variables to control the "version" of Buck2 that we want from S3.
          #
          # To upload the latest objects and get the hashes needed for this flake, run the update
          # script from the support directory via Buck2 itself. You can run that target again to
          # find the hashes, even if the objects have already been uploaded.

          releaseDate = "2023-06-29";
          hash = {
            "x86_64-linux"   = "sha256-uCIbLPms8Ees8SYFZgMuDBKcd2AVdRES/AOzkuCbl1o=";
            "aarch64-linux"  = "sha256-2f1MOzw+maXlw75Qfd9YIOF4447PuU+seHHGOJ2als0=";
            "x86_64-darwin"  = "sha256-UFez/5wucCv54riuhsQKCw9EfyUW7xdacbYTVi9ZQ1U=";
            "aarch64-darwin" = "sha256-pm9tf3aLllwq5ov3Fg9Umy25mpewGhBEjP5xSBzmFfQ=";
          }.${system};

          pname = "buck2";
          version = "unstable-${releaseDate}";

          nativeBuildInputs = with pkgs; [ zstd ];

          triple = {
            "x86_64-linux"   = "x86_64-unknown-linux-gnu";
            "aarch64-linux"  = "aarch64-unknown-linux-gnu";
            "x86_64-darwin"  = "x86_64-apple-darwin";
            "aarch64-darwin" = "aarch64-apple-darwin";
          }.${system};

          urlPrefix = "https://buck2-binaries.s3.us-east-2.amazonaws.com/${releaseDate}";
          archiveName = "buck2-${triple}.zst";

          src = pkgs.fetchurl {
            url = "${urlPrefix}/${archiveName}";
            sha256 = hash;
          };

          unpackPhase = ":";
          sourceRoot = ".";

          installPhase = ''
            mkdir -p $out/bin
            zstd -d ${src} -o buck2
            install -m755 -D buck2 $out/bin/buck2
          '';

          fixupPhase = if pkgs.stdenv.isLinux then ''
            patchelf \
              --set-interpreter "$(cat $NIX_CC/nix-support/dynamic-linker)" \
              $out/bin/buck2
          '' else
            "";
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

            # TODO(fnichol): we can remove these remaining packages when the
            # Make workflow is dropped
            automake
            coreutils
            gcc
            gnumake
            libtool
            pkg-config
            postgresql_14

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
