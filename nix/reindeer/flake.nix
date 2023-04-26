{
  description = "A builder for reindeer";

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
        mainBranchCommit = "86a03a1795bbf3feeaa2d1b48590ce71988974b2";
        gitHubSha256 = "sha256-+TDQoHhgvgq5QS0AuvUtAxZL0c+R4qHZfCvwh//GhIM=";

        reindeer = pkgs.rustPlatform.buildRustPackage rec {
          pname = "reindeer";
          version = mainBranchCommit;

          src = pkgs.fetchFromGitHub {
            owner = "facebookincubator";
            repo = pname;
            rev = mainBranchCommit;
            sha256 = gitHubSha256;
          };

          # TODO(nick,jacob): temporarily disable the check to investigate dependencies needed.
          doCheck = false;

          cargoPatches = [ ./Cargo.lock.patch ];
          cargoLock = { lockFile = ./Cargo.lock; };

          buildInputs = with pkgs;
            [ openssl ] ++ lib.optionals stdenv.isDarwin [
              libiconv
              darwin.apple_sdk.frameworks.Security
              darwin.apple_sdk.frameworks.CoreServices
            ];
        };
      in {
        defaultPackage = reindeer;
        packages.reindeer = reindeer;
      });
}
