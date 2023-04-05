{
  description = "Development environment for System Initiative";

  # Flake inputs
  inputs = {
    # Unstable nixpkgs to get pnpm
    nixpkgs.url = "github:NixOS/nixpkgs/master"; # also valid: "nixpkgs"
    rust-overlay.url = "github:oxalica/rust-overlay"; # A helper for Rust + Nix
  };

  # Flake outputs
  outputs = { self, nixpkgs, rust-overlay }:
    let
      # Overlays enable you to customize the Nixpkgs attribute set
      overlays = [
        # Makes a `rust-bin` attribute available in Nixpkgs
        (import rust-overlay)

        # Provides a `rustToolchain` attribute for Nixpkgs that we can use to 
        # create a Rust environment
        (self: super: {
          rustToolchain = super.rust-bin.fromRustupToolchainFile ./rust-toolchain;
        })

        # Override attributes from nixpkgs so that we can get a newer pnpm package
        (self: super: {
          si-overlay.pnpm = super.nodePackages.pnpm.override(oldAttrs: rec {
            version = "8.1.1";
            src = super.fetchurl {
              url = "https://registry.npmjs.org/pnpm/-/pnpm-8.1.1.tgz";
              sha512 = "XLzcc4O8YrqfQ1+qjPtHGDFcdUeno2Zk+kuuSc9CagIiY8y4uhnqQ2B7jW8tgwQDNmehewGZuqrAoskgCkbTnw==";
            };
          });
        })
      ];

      # Helper to provide system-specific attributes
      nameValuePair = name: value: { inherit name value; };
      genAttrs = names: f: builtins.listToAttrs (map (n: nameValuePair n (f n)) names);
      allSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = f: genAttrs allSystems (system: f {
        pkgs = import nixpkgs { inherit overlays system; };
      });
    in
    {
      # Development environment output
      devShells = forAllSystems ({ pkgs }: {
        default = pkgs.mkShell {
          # The Nix packages provided in the environment
          buildInputs = (with pkgs; [
            rustToolchain
            si-overlay.pnpm

            automake
            awscli
            bash
            butane
            coreutils
            docker-compose
            gcc
            git
            pgcli
            postgresql_14
            kubeval
            libtool
            gnumake
            protobuf
            skopeo
            jq
            nodejs-18_x

            nodePackages.typescript
            nodePackages.typescript-language-server
          ]) ++ pkgs.lib.optionals pkgs.stdenv.isDarwin (with pkgs; [
            libiconv
            pkgs.darwin.apple_sdk.frameworks.Security
          ]);
        };
      });
    };
}
