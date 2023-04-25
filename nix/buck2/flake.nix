{
  description = "buck2";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        # The commit is the primary variable controlling the version of buck2. When that variable
        # is changed, all variables in the below group should be re-evaluated and the update
        # script in this flake's directory should be ran with the commit passed in as the first
        # argument.
        latestBranchCommit = "c755dc088b5fc15b65f7b5537c34f7a90b228e2c";
        rustChannel = "nightly";
        rustVersion = "2023-01-24";
        gitHubSha256 = "sha256-V97Kq3TpEUDmmNfwPT3fA9pgYWS4UX2ykaXHMHc3hhE=";
        outputHashes = {
          "perf-event-0.4.8" =
            "sha256-4OSGmbrL5y1g+wdA+W9DrhWlHQGeVCsMLz87pJNckvw=";
          "tonic-0.8.3" = "sha256-xuQVixIxTDS4IZIN46aMAer3v4/81IQEG975vuNNerU=";
        };

        overlays = [ (import rust-overlay) ];

        pkgs = import nixpkgs { inherit system overlays; };

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
