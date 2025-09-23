{
  description = "System Initiative build and development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    #
    # NOTE 2025-07-17
    # ---------------
    #
    # Pin Deno to version 2.2.12 as this is the last version that successfully
    # built via Hydra. All newer versions revert to source-building which takes
    # way too long for every developer.
    #
    # When the build failures are clear and there are pre-built packages
    # available for Deno again, remove this input.
    #
    # References: https://github.com/NixOS/nixpkgs/issues/417331
    # References: https://github.com/NixOS/nixpkgs/pull/384838
    #
    # See: https://aalbacetef.io/blog/nix-pinning-a-specific-package-version-in-a-flake-using-overlays/
    # See: https://lazamar.co.uk/nix-versions/?channel=nixpkgs-unstable&package=deno
    pinnedDenoVersion.url = "github:NixOS/nixpkgs/4684fd6b0c01e4b7d99027a34c93c2e09ecafee2";
  };

  outputs = {
    nixpkgs,
    flake-utils,
    rust-overlay,
    pinnedDenoVersion,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      overlays = [
        (self: super: {
          nodejs = super.nodejs_20;
        })
        (import rust-overlay)
        # NOTE 2025-07-17: see inputs for explanation. When input is deleted,
        # remove this overlay too.
        (final: prev: {
          deno = pinnedDenoVersion.legacyPackages.${prev.system}.deno;
        })
      ];

      pkgs = import nixpkgs {inherit overlays system;};

      rustVersion = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain;
      rust-toolchain = rustVersion.override {
        extensions = ["clippy" "rust-analyzer" "rust-src"];
      };
      rustfmt-nightly = pkgs.rust-bin.nightly."2025-04-17".rustfmt;

      nodePkgs = pkgs.nodePackages.override {
        nodejs = pkgs.nodejs_20;
      };


      buck2NativeBuildInputs = with pkgs;
        [
          b3sum
          buck2
          deno
          gh
          gitMinimal
          makeWrapper
          pnpm
          nodejs
          python3
          ripgrep
          rust-toolchain
          rustfmt-nightly
          taplo

          # breakpointHook
        ]
        ++ lib.optionals pkgs.stdenv.isDarwin [
          pkgs.darwin.sigtool
        ];

      buck2BuildInputs = with pkgs;
        []
        ++ lib.optionals pkgs.stdenv.isLinux [
          clang
          glibc
          glibc.libgcc
          lvm2
          llvmPackages.libclang.lib
        ]
        ++ lib.optionals pkgs.stdenv.isDarwin [
          libiconv
        ];

      # The file name of the program interpreter/dynamic linker. We're primarily
      # interested in the Linux system values.
      interpreterName =
        {
          "x86_64-linux" = "ld-linux-x86-64.so.2";
          "aarch64-linux" = "ld-linux-aarch64.so.1";
          "x86_64-darwin" = "/dev/null";
          "aarch64-darwin" = "/dev/null";
        }
        .${
          system
        };

      # This isn't an exact science, but confirmed the system interpreter by
      # running `ldd /bin/sh` in Docker containers running:
      # - debian:9-slim
      # - ubuntu:rolling
      # - fedora:38
      # - archlinux:latest
      # - amazonlinux:latest
      # - rockylinux:9-minimal
      systemInterpreter =
        {
          "x86_64-linux" = "/lib64/ld-linux-x86-64.so.2";
          "aarch64-linux" = "/lib/ld-linux-aarch64.so.1";
          "x86_64-darwin" = "/dev/null";
          "aarch64-darwin" = "/dev/null";
        }
        .${
          system
        };

      langJsExtraPkgs = with pkgs; [
        awscli2
        azure-cli
        butane
        doctl
        fastly
        flyctl
        google-cloud-sdk
        govc
        linode-cli
        minio-client
        openssh
        skopeo
        (pkgs.python3.withPackages (p:
          with p; [
            cfn-lint
          ]))
      ];

    in
      with pkgs; rec {
        devShells = {
          # Full development environment
          default = mkShell {
            shellHook = with pkgs; if pkgs.stdenv.isLinux then ''
              export LIBCLANG_PATH="${pkgs.llvmPackages.libclang.lib}/lib"
              export BINDGEN_EXTRA_CLANG_ARGS="\
                $(< ${pkgs.stdenv.cc}/nix-support/libc-crt1-cflags) \
                $(< ${pkgs.stdenv.cc}/nix-support/libc-cflags) \
                $(< ${pkgs.stdenv.cc}/nix-support/cc-cflags) \
                $(< ${pkgs.stdenv.cc}/nix-support/libcxx-cxxflags) \
                $NIX_CFLAGS_COMPILE"
              export OUT=${placeholder "out"}
              echo $OUT
            '' else "";
            packages =
              [
                alejandra
                buildkite-test-collector-rust
                cargo-insta
                cargo-sort
                docker-compose
                jq
                nats-top
                natscli
                openapi-generator-cli
                # pgcli
                reindeer
                shellcheck
                shfmt
                spicedb-zed
                taplo
                tilt
                tokio-console
                typos
                yapf
              ]
              ++ buck2NativeBuildInputs
              ++ buck2BuildInputs
              ++ langJsExtraPkgs;
          };
        };

        packages = {
          ci-tools = pkgs.symlinkJoin {
            name = "ci-tools";
            paths =
              [
                awscli2
                buildkite-test-collector-rust
                cargo-sort
                deno
                docker
                docker-compose
                gh
                jq
                shfmt
                shellcheck
                yapf
              ]
              ++ buck2BuildInputs
              ++ buck2NativeBuildInputs;
            };
          };

        formatter = alejandra;
      });
}
