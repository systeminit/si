{
  description = "Development environment for System Initiative";

  # Flake inputs
  inputs = {
    # rust-overlay is designed to work with nixos-unstable
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
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

        # Context: https://github.com/NixOS/nixpkgs/issues/226677
        buck2 = pkgs.stdenv.mkDerivation rec {
          # These are the primary variables to control the "version" of buck2 that we want from S3.
          #
          # To upload the latest objects and get the hashes needed for this flake, run
          # "buck2 run //support:upload-latest-buck2". You can run that target again to find the
          # hashes, even if the objects have already been uploaded.
          #
          # For the hashes, default to "aarch64-darwin".
          buckDate = "2023-05-31";
          buckSha256 = if system == "x86_64-linux" then
            "sha256-0MzhwUgBO7BUuwExGx/htm6UOxPr00CKC9edNYqwQoA="
          else if system == "aarch64-linux" then
            "sha256-/Fo6IEPx6+5H4/kHxPahGUJlMxSeS5mXWtHfux4PxL4="
          else if system == "x86_64-darwin" then
            "sha256-UzhZznPbwzYTNLUhLtW7uNC2F7R6UMSFonDY8StunBc="
          else
            "sha256-i2MVcLcrGN3G4tE0aaf/65zBJeLBkSgm8vroQ1EenlA=";

          pname = "buck2";
          version = "unstable-${buckDate}";

          # Default to "aarch64-darwin" for the system in the URL. Translate to how the buck2 team
          # names their os/arch combinations.
          buckSystem = if system == "x86_64-linux" then
            "x86_64-unknown-linux-gnu"
          else if system == "aarch64-linux" then
            "aarch64-unknown-linux-gnu"
          else if system == "x86_64-darwin" then
            "x86_64-apple-darwin"
          else
            "aarch64-apple-darwin";

          buckObject = pkgs.fetchurl {
            url =
              "https://buck2-binaries.s3.us-east-2.amazonaws.com/${buckDate}/buck2-${buckSystem}.zst";
            sha256 = buckSha256;
          };

          unpackPhase = ":";
          sourceRoot = ".";

          nativeBuildInputs = with pkgs; [ zstd ];

          installPhase = ''
            mkdir -p $out/bin
            zstd -d ${buckObject} -o buck2
            install -m755 -D buck2 $out/bin/buck2
          '';
        };

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
