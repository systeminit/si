ARG BUILDER_IMAGE=cache
ARG RUST_VERSION=1.57.0

ARG BUILDER_LANG_JS_IMAGE=cache_lang_js
ARG NODE_VERSION=16.13.1

###########################################################################
# Cache Stage
###########################################################################
# https://hub.docker.com/_/rust
FROM rust:$RUST_VERSION as cache
ARG RUST_VERSION
ARG BIN=veritech

# hadolint ignore=DL3008
RUN set -eux; \
    apt-get update; \
    apt-get install --yes --no-install-recommends \
        lld \
        protobuf-compiler \
    ;
WORKDIR /usr/src
COPY . .

RUN set -eux; \
    cargo build --release --locked --bin $BIN --bin cyclone;

RUN set -eux; \
    find . -maxdepth 1 ! -path './target' -and ! -path '.' -exec rm -rf {} \;;

###########################################################################
# Builder Stage
###########################################################################
# hadolint ignore=DL3006
FROM $BUILDER_IMAGE as builder
ARG BIN=veritech

WORKDIR /usr/src
COPY . .

RUN set -eux; \
    cargo build --release --locked --bin $BIN --bin cyclone;

# Create a 2-part executable which occupies less space in RAM but still links
# to debugging information for in-production debugging
# hadolint ignore=DL3003
RUN set -eux; \
    objcopy --only-keep-debug target/release/$BIN target/release/$BIN.dbg; \
    objcopy --strip-debug target/release/$BIN; \
    (cd target/release && objcopy --add-gnu-debuglink=$BIN.dbg $BIN);

# hadolint ignore=DL3003
RUN set -eux; \
    objcopy --only-keep-debug target/release/cyclone target/release/cyclone.dbg; \
    objcopy --strip-debug target/release/cyclone; \
    (cd target/release && objcopy --add-gnu-debuglink=cyclone.dbg cyclone);

###########################################################################
# lang-js Cache Stage
###########################################################################
FROM node:$NODE_VERSION as cache_lang_js
ARG NODE_VERSION

WORKDIR /usr/src
COPY bin/lang-js/package.json bin/lang-js/package-lock.json ./

RUN set -eux; \
    npm ci;

RUN set -eux; \
    find . \
        -maxdepth 1 \
        ! -path './node_modules' \
        -and \
        ! -path '.' \
        -exec rm -rf {} \;;

###########################################################################
# lang-js Builder Stage
###########################################################################
# hadolint ignore=DL3006
FROM $BUILDER_LANG_JS_IMAGE as builder_lang_js

WORKDIR /usr/src
COPY bin/lang-js .

RUN set -eux; \
    env; \
    npm run package; \
    ls -la target;

###########################################################################
# Final Stage
###########################################################################
FROM debian:stable-slim as final
ARG BIN=veritech

RUN set -eux; \
    useradd -ms /bin/bash app; \
    for dir in /run /etc /usr/local/etc /home/app/.config; do \
        mkdir -pv "$dir/$BIN"; \
    done;

# hadolint ignore=DL3008
RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        busybox \
        ca-certificates \
        skopeo \
        wget \
    ; \
    rm -rf /var/lib/apt/lists/*

# https://github.com/instrumenta/kubeval/releases
ARG KUBEVAL_VERSION=0.16.1

# https://github.com/coreos/butane/releases
ARG BUTANE_VERSION=0.16.0

# hadolint ignore=DL3047
RUN set -eux; \
    wget -O /tmp/kubeval.tar.gz \
        "https://github.com/instrumenta/kubeval/releases/download/v$KUBEVAL_VERSION/kubeval-linux-amd64.tar.gz"; \
    tar xvf /tmp/kubeval.tar.gz -C /usr/local/bin kubeval; \
    rm -v /tmp/kubeval.tar.gz; \
    wget -O /usr/local/bin/butane \
        "https://github.com/coreos/butane/releases/download/v$BUTANE_VERSION/butane-x86_64-unknown-linux-gnu"; \
    chmod +x /usr/local/bin/butane; \
    wget -O /tmp/awscliv2.zip \
        "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip"; \
    busybox unzip /tmp/awscliv2.zip -d /tmp; \
    /tmp/aws/install; \
    rm -rf /tmp/aws /tmp/awscliv2.zip;

WORKDIR /run/$BIN
COPY --from=builder /usr/src/target/release/$BIN /usr/local/bin/
COPY --from=builder /usr/src/target/release/$BIN.dbg /usr/local/bin/.debug/

COPY --from=builder /usr/src/target/release/cyclone /usr/local/bin/
COPY --from=builder /usr/src/target/release/cyclone.dbg /usr/local/bin/.debug/

COPY --from=builder_lang_js /usr/src/target/lang-js /usr/local/bin/

EXPOSE 5156/tcp
ENTRYPOINT [ \
  "/sbin/runuser", "-u", "app", "--", "/usr/local/bin/veritech" \
]
