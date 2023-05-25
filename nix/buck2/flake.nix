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
          # The primary variable to control the "version" of buck2 that we want from S3.
          date = "2023-05-24";

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

          # Default to "aarch64-darwin" for the sha256 value.
          binarySha256 = if system == "x86_64-linux" then
            "sha256-TzKQM7P9zdcUx/RAjATlKYML2bzMPSe4wZNw6kupQS8="
          else if system == "aarch64-linux" then
            "sha256-De/1MxFs1u8c83a1VltGqXjEwwS+2apSY1IXvX9EaFQ="
          else if system == "x86_64-darwin" then
            "sha256-WP+LivDDAlgZ8UBcUadkW6+AxseRqHXGX9rOFykNEz4="
          else
            "sha256-tpkfodVXP3C66ZR/NSjMrdyJrqg5h0NYO5l9B0TZFOg=";

          # Uncomment these two variables (and comment the ones of the same name) to update the
          # sha256 values for each system.
          # binarySystem = "aarch64-apple-darwin";
          # binarySha256 = pkgs.lib.fakeSha256;

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
