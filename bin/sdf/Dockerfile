ARG BUILDER_IMAGE=cache
ARG RUST_VERSION=1.57.0

###########################################################################
# Cache Stage
###########################################################################
FROM rust:$RUST_VERSION as cache
ARG RUST_VERSION
ARG BIN=sdf

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
    cargo build --release --locked --bin $BIN;

RUN set -eux; \
    find . -maxdepth 1 ! -path './target' -and ! -path '.' -exec rm -rf {} \;;

###########################################################################
# Builder Stage
###########################################################################
# hadolint ignore=DL3006
FROM $BUILDER_IMAGE as builder
ARG BIN=sdf

WORKDIR /usr/src
COPY . .

RUN set -eux; \
    cargo build --release --locked --bin $BIN;

# Create a 2-part executable which occupies less space in RAM but still links
# to debugging information for in-production debugging
# hadolint ignore=DL3003
RUN set -eux; \
    objcopy --only-keep-debug target/release/$BIN target/release/$BIN.dbg; \
    objcopy --strip-debug target/release/$BIN; \
    (cd target/release && objcopy --add-gnu-debuglink=$BIN.dbg $BIN);

###########################################################################
# Final Stage
###########################################################################
FROM debian:stable-slim as final
ARG BIN=sdf

RUN set -eux; \
    useradd -ms /bin/bash app; \
    for dir in /run /etc /usr/local/etc /home/app/.config; do \
        mkdir -pv "$dir/$BIN"; \
    done;

WORKDIR /run/$BIN
COPY --from=builder /usr/src/target/release/$BIN /usr/local/bin/
COPY --from=builder /usr/src/target/release/$BIN.dbg /usr/local/bin/.debug/

EXPOSE 5156/tcp
ENTRYPOINT [ \
  "/sbin/runuser", "-u", "app", "--", "/usr/local/bin/sdf" \
]
