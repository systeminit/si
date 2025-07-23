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
          bash
          buck2
          cacert
          clang
          deno
          gitMinimal
          lld
          makeWrapper
          minica
          nodePkgs.pnpm
          nodejs
          protobuf
          python3
          ripgrep
          rust-toolchain
          rustfmt-nightly

          # breakpointHook
        ]
        ++ lib.optionals pkgs.stdenv.isDarwin [
          darwin.sigtool
        ];

      buck2BuildInputs = with pkgs;
        []
        ++ lib.optionals pkgs.stdenv.isLinux [
          glibc
          glibc.libgcc
          lvm2
          llvmPackages.libclang.lib
        ]
        ++ lib.optionals pkgs.stdenv.isDarwin [
          libiconv
          darwin.apple_sdk.frameworks.Security
          darwin.apple_sdk.frameworks.SystemConfiguration
          darwin.apple_sdk.frameworks.CoreFoundation
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
        butane
        gh
        skopeo
        openssh
        minio-client
        fastly
        doctl
        google-cloud-sdk
        linode-cli
        flyctl
        azure-cli
        govc
        (pkgs.python3.withPackages (p:
          with p; [
            cfn-lint
          ]))
        deno
      ];

      standaloneBinaryDerivation = {
        pkgName,
        fromPkg,
        bin ? pkgs.lib.strings.removeSuffix "-standalone" pkgName,
      }:
        pkgs.stdenv.mkDerivation {
          name = pkgName;
          __impure = true;
          src = ./.;
          buildInputs = [fromPkg];
          installPhase = ''
            install -Dv "${fromPkg}/bin/${bin}" "$out/bin/${bin}"
          '';
          postFixup =
            ""
            + pkgs.lib.optionalString (pkgs.stdenv.isDarwin) ''
              nix_lib="$(otool -L "$out/bin/$name" \
                | grep libiconv.dylib \
                | awk '{print $1}'
              )"
              install_name_tool \
                -change \
                "$nix_lib" \
                /usr/lib/libiconv.2.dylib \
                "$out/bin/$name" \
                2>/dev/null
            ''
            + pkgs.lib.optionalString (pkgs.stdenv.isLinux) ''
              patchelf \
                --set-interpreter "${systemInterpreter}" \
                --remove-rpath \
                "$out/bin/${bin}"
            '';
          dontPatchELF = true;
          dontAutoPatchELF = true;
        };

      buck2Derivation = {
        pathPrefix,
        pkgName,
        extraBuildInputs ? [],
        stdBuildPhase ? ''
          buck2 build \
            @//mode/release \
            "$buck2_target" \
            --verbose 8 \
            --out "build/$name-$system"
        '',
        extraBuildPhase ? "",
        installPhase,
        dontStrip ? true,
        dontPatchELF ? false,
        dontAutoPatchELF ? false,
        postFixup ? "",
      }:
        pkgs.stdenv.mkDerivation {
          name = pkgName;
          buck2_target = "//${pathPrefix}/${pkgName}";
          __impure = true;
          src = ./.;
          nativeBuildInputs =
            buck2NativeBuildInputs
            ++ pkgs.lib.optionals (
              pkgs.stdenv.isLinux && !dontAutoPatchELF
            ) [pkgs.autoPatchelfHook];
          buildInputs = buck2BuildInputs ++ extraBuildInputs;
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
          configurePhase = ''
            export LIBCLANG_PATH="${pkgs.llvmPackages.libclang.lib}/lib"
            export BINDGEN_EXTRA_CLANG_ARGS="\
              $(< ${pkgs.stdenv.cc}/nix-support/libc-crt1-cflags) \
              $(< ${pkgs.stdenv.cc}/nix-support/libc-cflags) \
              $(< ${pkgs.stdenv.cc}/nix-support/cc-cflags) \
              $(< ${pkgs.stdenv.cc}/nix-support/libcxx-cxxflags) \
              $NIX_CFLAGS_COMPILE"
            export OUT=${placeholder "out"}
            echo $OUT
          '';
          buildPhase =
            ''
              export HOME="$(dirname $(pwd))/home"
              mkdir -p build
            ''
            + stdBuildPhase
            + extraBuildPhase;
          inherit installPhase;
          inherit dontStrip;
          inherit dontPatchELF;
          inherit postFixup;
        };

      appDerivation = {
        pkgName,
        extraBuildPhase ? "",
        extraInstallPhase ? "",
        dontStrip ? false,
        dontPatchELF ? false,
        dontAutoPatchELF ? false,
      }: (buck2Derivation {
        pathPrefix = "app";
        inherit pkgName;
        inherit extraBuildPhase;
        installPhase =
          ''
            mkdir -pv "$out"
            cp -rpv "build/$name-$system" "$out/html"
          ''
          + extraInstallPhase;
        inherit dontStrip;
        inherit dontPatchELF;
        inherit dontAutoPatchELF;
      });

      binDerivation = {
        pkgName,
        extraBuildPhase ? "",
        extraInstallPhase ? "",
        dontStrip ? true,
        dontPatchELF ? false,
        dontAutoPatchELF ? false,
      }: (buck2Derivation {
        pathPrefix = "bin";
        inherit pkgName;
        inherit extraBuildPhase;
        installPhase =
          ''
            mkdir -pv "$out/bin"
            cp -pv "build/$name-$system" "$out/bin/$name"
          ''
          + extraInstallPhase;
        inherit dontStrip;
        inherit dontPatchELF;
        inherit dontAutoPatchELF;
      });
    in
      with pkgs; rec {
        packages = {
          auth-api = binDerivation {pkgName = "auth-api";};

          cyclone = binDerivation {pkgName = "cyclone";};

          # autoPatchingElf and stripping will break deno compile'd binaries.
          # We need to make sure we know where glibc and friends exist and since
          # we can't patchelf, we need to ensure we drop the dynmic linker in a
          # known place. Note that LD_LIBRAY_PATH is unset when using siExec to
          # ensure the binaries our binaries run don't inherit it.
          lang-js = binDerivation {
            pkgName = "lang-js";
            dontAutoPatchELF = true;
            dontStrip = true;
            extraBuildPhase = ''
              export DENO_DIR="$TMPDIR/deno-cache"
              mkdir -p "$DENO_DIR"

              # Cache the dependencies
              ${pkgs.deno}/bin/deno cache \
                --node-modules-dir \
                --reload \
                bin/lang-js/src/sandbox.ts \

            '';
            extraInstallPhase = ''
              # build a cache of our deps so we don't need to donwload them in
              # the firecracker jail each time
              mkdir -p $out/cache
              cp -rL "$TMPDIR/deno-cache"/* $out/cache/
              chmod -R 755 $out/cache

              # since we can't patchelf, we need to ensure the dynamic linker
              # is where we expect it. This gets droppped into the rootfs as
              # /lib64/ld-linux-x86-64.so.2 -> /nix/store/*/ld-linux-x86-64.20.2
              mkdir -p $out/lib64
              ln -sf \
                "${pkgs.glibc}/lib/${interpreterName}" \
                "$out/lib64/${interpreterName}"

              wrapProgram $out/bin/lang-js \
                --set LD_LIBRARY_PATH "${pkgs.lib.makeLibraryPath [
                pkgs.glibc
                pkgs.gcc-unwrapped.lib
              ]}" \
                --set DENO_DIR "$out/cache" \
                --prefix PATH : ${pkgs.lib.makeBinPath langJsExtraPkgs} \
                --run 'cd "$(dirname "$0")"'
                # ^ deno falls over trying to resolve libraries if you don't set
                # the working path
            '';
          };

          bedrock = binDerivation {pkgName = "bedrock";};

          edda = binDerivation {pkgName = "edda";};

          forklift = binDerivation {pkgName = "forklift";};

          innit = binDerivation {pkgName = "innit";};

          innitctl = binDerivation {pkgName = "innitctl";};

          luminork = binDerivation {pkgName = "luminork";};

          module-index = binDerivation {pkgName = "module-index";};

          pinga = binDerivation {pkgName = "pinga";};

          rebaser = binDerivation {pkgName = "rebaser";};

          sdf = binDerivation {pkgName = "sdf";};

          si-fs = binDerivation {pkgName = "si-fs";};

          si-fs-standalone = standaloneBinaryDerivation {
            pkgName = "si-fs-standalone";
            fromPkg = packages.si-fs;
          };

          veritech = binDerivation {pkgName = "veritech";};

          web = appDerivation rec {
            pkgName = "web";
            extraBuildPhase = ''
              buck2 build app/web:nginx_src --verbose 3 --out build/nginx
              buck2 build app/web:docker-entrypoint.sh \
                --verbose 3 --out build/docker-entrypoint.sh
            '';
            extraInstallPhase = ''
              patchShebangs --host build/docker-entrypoint.sh
              substituteInPlace build/docker-entrypoint.sh \
                --replace @@nginx@@ "${nginx}/bin/nginx" \
                --replace @@conf@@ "$out/conf/nginx.conf" \
                --replace @@prefix@@ "$out"

              mkdir -pv "$out/bin" "$out/conf"
              cp -pv build/nginx/nginx.conf "$out/conf/nginx.conf"
              cp -pv "${nginx}/conf/mime.types" "$out/conf"/
              cp -pv build/docker-entrypoint.sh "$out/bin/${pkgName}"
            '';
          };
        };

        devShells.default = mkShell {
          # Env Vars so bindgen can find libclang
          shellHook = ''
            export LIBCLANG_PATH="${pkgs.llvmPackages.libclang.lib}/lib"
            export BINDGEN_EXTRA_CLANG_ARGS="\
              $(< ${pkgs.stdenv.cc}/nix-support/libc-crt1-cflags) \
              $(< ${pkgs.stdenv.cc}/nix-support/libc-cflags) \
              $(< ${pkgs.stdenv.cc}/nix-support/cc-cflags) \
              $(< ${pkgs.stdenv.cc}/nix-support/libcxx-cxxflags) \
              $NIX_CFLAGS_COMPILE"
            export OUT=${placeholder "out"}
            echo $OUT
          '';
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
            # Directly add the build dependencies for the packages rather than
            # use: `inputsFrom = lib.attrValues packages;`.
            #
            # This tweak means our flake doesn't require `impure-derivations`
            # and `ca-derivations` experimental features by default--only when
            # attempting to build the packages directly.
            ++ buck2NativeBuildInputs
            ++ buck2BuildInputs
            ++ langJsExtraPkgs;
        };

        formatter = alejandra;
      });
}
