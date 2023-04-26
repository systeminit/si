{
  description = "A builder for buck2";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        # First, run the update script with the new commit. Then, use "pkgs.lib.fakeSha256" to
        # determine hashes.
        latestBranchCommit = "6ce606949631ca86c4cbb3de53ee90ceb031257a";
        rustChannel = "nightly";
        rustVersion = "2023-03-07";
        gitHubSha256 = "sha256-gsmGC3qYnmVAqV7Hnqt0OlQ9ek4ODenhDH5GN1czk+Q=";
        outputHashes = {
          "perf-event-0.4.8" =
            "sha256-4OSGmbrL5y1g+wdA+W9DrhWlHQGeVCsMLz87pJNckvw=";
          "tonic-0.8.3" = "sha256-xuQVixIxTDS4IZIN46aMAer3v4/81IQEG975vuNNerU=";
        };

        buck2RustPlatform = pkgs.makeRustPlatform {
          rustc = pkgs.rust-bin."${rustChannel}"."${rustVersion}".default;
          cargo = pkgs.rust-bin."${rustChannel}"."${rustVersion}".default;
        };

        buck2 = buck2RustPlatform.buildRustPackage rec {
          pname = "buck2";
          version = latestBranchCommit;

          src = pkgs.fetchFromGitHub {
            owner = "facebook";
            repo = pname;
            rev = latestBranchCommit;
            sha256 = gitHubSha256;
          };

          # TODO(nick,jacob): temporarily disable the check to investigate dependencies needed.
          doCheck = false;

          cargoPatches = [ ./Cargo.lock.patch ];
          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = outputHashes;
          };

          nativeBuildInputs = with pkgs; [ pkg-config protobuf ];
          buildInputs = with pkgs;
            [ openssl ] ++ lib.optionals stdenv.isDarwin [
              libiconv
              darwin.apple_sdk.frameworks.Security
              darwin.apple_sdk.frameworks.CoreServices
            ];

          BUCK2_BUILD_PROTOC = "${pkgs.protobuf}/bin/protoc";
          BUCK2_BUILD_PROTOC_INCLUDE = "${pkgs.protobuf}/include";
        };
      in {
        defaultPackage = buck2;
        packages.buck2 = buck2;
      });
}
