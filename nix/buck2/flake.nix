{
  description = "A downloader for buck2 pre-compiled binaries";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };

        buck2 = pkgs.stdenv.mkDerivation rec {
          # The primary variables to control the "version" of buck2 that we want from S3.
          #
          # To upload the latest objects and get the hashes needed for this flake, run
          # "buck2 run //support:upload-latest-buck2". You can run that target again to find the
          # hashes, even if the objects have already been uploaded.
          #
          # For the hashes, default to "aarch64-darwin".
          date = "2023-05-31";
          binarySha256 = if system == "x86_64-linux" then
            "sha256-0MzhwUgBO7BUuwExGx/htm6UOxPr00CKC9edNYqwQoA="
          else if system == "aarch64-linux" then
            "sha256-/Fo6IEPx6+5H4/kHxPahGUJlMxSeS5mXWtHfux4PxL4="
          else if system == "x86_64-darwin" then
            "sha256-UzhZznPbwzYTNLUhLtW7uNC2F7R6UMSFonDY8StunBc="
          else
            "sha256-i2MVcLcrGN3G4tE0aaf/65zBJeLBkSgm8vroQ1EenlA=";

          pname = "buck2";
          version = "unstable-${date}";

          # Default to "aarch64-darwin" for the system in the URL. Translate to how the buck2 team
          # names their os/arch combinations.
          binarySystem = if system == "x86_64-linux" then
            "x86_64-unknown-linux-gnu"
          else if system == "aarch64-linux" then
            "aarch64-unknown-linux-gnu"
          else if system == "x86_64-darwin" then
            "x86_64-apple-darwin"
          else
            "aarch64-apple-darwin";

          binary = pkgs.fetchurl {
            url =
              "https://buck2-binaries.s3.us-east-2.amazonaws.com/${date}/buck2-${binarySystem}.zst";
            sha256 = binarySha256;
          };

          unpackPhase = ":";
          sourceRoot = ".";

          nativeBuildInputs = with pkgs; [ zstd ];

          installPhase = ''
            mkdir -p $out/bin
            zstd -d ${binary} -o buck2
            install -m755 -D buck2 $out/bin/buck2
          '';
        };
      in {
        defaultPackage = buck2;
        packages.buck2 = buck2;
      });
}
