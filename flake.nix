{
  description = "Development environment for System Initiative";

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

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      overlays = [
        (self: super: {
          nodejs = super.nodejs-18_x;
          pnpm = super.nodePackages.pnpm;
        })
        (import rust-overlay)
      ];

      pkgs = import nixpkgs {inherit overlays system;};

      rustVersion = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain;
      rust-toolchain = rustVersion.override {
        extensions = ["rust-analyzer" "rust-src"];
      };

      buck2NativeBuildInputs = with pkgs;
        [
          bash
          buck2
          cacert
          clang
          gitMinimal
          lld
          nodejs
          pnpm
          protobuf
          python3
          ripgrep
          rust-toolchain

          # breakpointHook
        ]
        ++ lib.optionals pkgs.stdenv.isDarwin [
          darwin.sigtool
        ]
        ++ lib.optionals pkgs.stdenv.isLinux [
          autoPatchelfHook
        ];

      buck2BuildInputs = with pkgs;
        []
        ++ lib.optionals pkgs.stdenv.isLinux [
          glibc
          glibc.libgcc
        ]
        ++ lib.optionals pkgs.stdenv.isDarwin [
          libiconv
          darwin.apple_sdk.frameworks.Security
          darwin.apple_sdk.frameworks.SystemConfiguration
        ];

      buck2Derivation = {
        pathPrefix,
        pkgName,
      }:
        pkgs.stdenv.mkDerivation rec {
          name = pkgName;
          buck2_target = "//${pathPrefix}/${pkgName}";
          __impure = true;
          src = ./.;
          nativeBuildInputs = buck2NativeBuildInputs;
          buildInputs = buck2BuildInputs;
          runtimeDependencies = map pkgs.lib.getLib buck2BuildInputs;
          postPatch = with pkgs; ''
            rg -l '#!(/usr/bin/env|/bin/bash|/bin/sh)' prelude prelude-si \
              | while read -r file; do
                patchShebangs --build "$file"
              done

            rg -l '(/usr/bin/env|/bin/bash)' prelude prelude-si \
              | while read -r file; do
                substituteInPlace "$file" \
                  --replace /usr/bin/env "${coreutils}/bin/env" \
                  --replace /bin/bash "${bash}/bin/bash"
              done
          '';
          buildPhase = ''
            export HOME="$(dirname $(pwd))/home"
            buck2 build "$buck2_target" --verbose 3 --out "$name-$system"
          '';
          installPhase = ''
            mkdir -p "$out/bin"
            cp -p "$name-$system" "$out/bin/$name"
          '';
        };

      binDerivation = pkgName: (buck2Derivation {
        pathPrefix = "bin";
        inherit
          pkgName
          ;
      });
    in
      with pkgs; rec {
        packages = {
          council = binDerivation "council";
          cyclone = binDerivation "cyclone";
          lang-js = binDerivation "lang-js";
          module-index = binDerivation "module-index";
          pinga = binDerivation "pinga";
          sdf = binDerivation "sdf";
          si = binDerivation "si";
          veritech = binDerivation "veritech";
        };

        devShells.default = mkShell {
          packages =
            [
              alejandra
              docker-compose
              gh
              jq
              pgcli
              reindeer
              tilt

              awscli
              butane
              kubeval
              skopeo
            ]
            # Directly add the build depndencies for the packages rather than
            # use: `inputsFrom = lib.attrValues packages;`.
            #
            # This tweak means our flake doesn't require `impure-derivations`
            # and `ca-derivations` experimental features by default--only when
            # attempting to build the packages directly.
            ++ buck2NativeBuildInputs
            ++ buck2BuildInputs;
        };

        formatter = alejandra;
      });
}
