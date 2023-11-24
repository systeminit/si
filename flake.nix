{
  description = "System Initiative development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    # Pin awscli2 to version 2.13.25 until unstable channel has updated pkg
    # resolving build error.
    #
    # References: https://github.com/NixOS/nixpkgs/issues/268737
    # References: https://github.com/NixOS/nixpkgs/pull/268590
    # See: https://lazamar.co.uk/nix-versions/?channel=nixpkgs-unstable&package=awscli2
    awscli2-pin-pkgs.url = "https://github.com/NixOS/nixpkgs/archive/9957cd48326fe8dbd52fdc50dd2502307f188b0d.tar.gz";
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
    awscli2-pin-pkgs,
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
      awscli2-pin = import awscli2-pin-pkgs {inherit system;};

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
          makeWrapper
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

      # This isn't an exact science, but confirming the system interpreter by
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
        .${system};

      langJsExtraPkgs = with pkgs; [
        awscli2-pin.awscli2
        butane
        gh
        skopeo
      ];

      buck2Derivation = {
        pathPrefix,
        pkgName,
        extraBuildInputs ? [],
        stdBuildPhase ? ''
          buck2 build "$buck2_target" --verbose 8 --out "build/$name-$system"
        '',
        extraBuildPhase ? "",
        installPhase,
        dontStrip ? false,
        dontPatchELF ? false,
        dontAutoPatchELF ? false,
        postFixup ? "",
      }:
        pkgs.stdenv.mkDerivation rec {
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
        extraInstallPhase ? "",
        dontStrip ? false,
        dontPatchELF ? false,
        dontAutoPatchELF ? false,
      }: (buck2Derivation {
        pathPrefix = "bin";
        inherit pkgName;
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
          council = binDerivation {pkgName = "council";};

          cyclone = binDerivation {pkgName = "cyclone";};

          # This one's awful: we don't have a stanalone binary here, we have a
          # directory of stuff with a `bin/$name` entrypoint shell script to
          # invoke this beast. Additionally there are `node_modules` symlinks
          # everywhere and Buck2's normal `buck2 build ... --out ...` option
          # dies on some symlinks referring to directories and not files.
          # Instead we'll have to parse a build report JSON to get the output
          # path and copy it ourselves.
          lang-js = buck2Derivation {
            pathPrefix = "bin";
            pkgName = "lang-js";
            extraBuildInputs = [pkgs.jq];
            stdBuildPhase = ''
              buck2 build "$buck2_target" --verbose 8 --build-report report.log
              dist_dir="$(jq -r \
                '.results | to_entries | map(.value)[0].outputs.DEFAULT[0]' \
                <report.log
              )"
              cp -rpv "$dist_dir" "build/$name-$system"
            '';
            installPhase = ''
              mkdir -pv "$out"
              cp -rpv "build/$name-$system"/* "$out"/
              mv -v "$out/bin/lang-js" "$out/bin/.lang-js"
              # Need to escape this shell variable which should not be
              # iterpreted in Nix as a variable nor a shell variable when run
              # but rather a literal string which happens to be a shell
              # variable. Nuclear arms race of quoting and escaping special
              # characters to make this work...
              substituteInPlace "$out/bin/.lang-js" \
                --replace "#!${pkgs.coreutils}/bin/env sh" "#!${pkgs.bash}/bin/sh" \
                --replace "\''${0%/*}/../lib/" "$out/lib/" \
                --replace "exec node" "exec ${pkgs.nodejs}/bin/node"
              makeWrapper "$out/bin/.lang-js" "$out/bin/lang-js" \
                --prefix PATH : ${pkgs.lib.makeBinPath langJsExtraPkgs}
            '';
          };

          module-index = binDerivation {pkgName = "module-index";};

          pinga = binDerivation {pkgName = "pinga";};

          sdf = binDerivation {pkgName = "sdf";};

          si = binDerivation {pkgName = "si";};

          # Builds a standalone binary that is divorced from the Nix build and
          # runtime environment
          si-binary = buck2Derivation {
            pathPrefix = "bin";
            pkgName = "si";
            stdBuildPhase = ''
              # TODO(fnichol): we currently have a dynamically linked library of
              # libc++abi being looked for in a `/nix/store` path when this
              # binary is compiled on macOS, so for the meantime until this is
              # fixed, we're going to perform a Cargo build of the binary (it
              # does not suffer from this issue).
              cargo build --bin "$name" --release
              cp -pv "target/release/$name" "build/$name-$system"
            '';
            installPhase = ''
              mkdir -pv "$out/bin"
              cp -pv "build/$name-$system" "$out/bin/$name"
            '';
            dontPatchELF = true;
            dontAutoPatchELF = true;
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
                  "$out/bin/$name"
              ''
              + ''
                mkdir -pv "$out/tarballs"
                tar cvf "$out/tarballs/$name-$system.tar" -C "$out/bin" "$name"
                gzip -9 "$out/tarballs/$name-$system.tar"
              '';
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
          packages =
            [
              alejandra
              docker-compose
              jq
              # pgcli
              reindeer
              shellcheck
              shfmt
              tilt
              typos
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
