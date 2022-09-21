ARG BUILDER_IMAGE=cache
ARG RUST_VERSION=1.57.0

###########################################################################
# Cache Stage
###########################################################################
FROM rust:$RUST_VERSION as cache
ARG RUST_VERSION
ARG BIN=sdf

RUN apt-get update && apt-get install -y lld protobuf-compiler
WORKDIR /usr/src
COPY . .

RUN cargo build --release --bin $BIN
RUN find . -maxdepth 1 ! -path './target' -and ! -path '.' -exec rm -rf {} \;

###########################################################################
# Builder Stage
###########################################################################
FROM $BUILDER_IMAGE as builder
ARG BIN=sdf

WORKDIR /usr/src
COPY . .

RUN cargo build --release --bin $BIN
# Create a 2-part executable which occupies less space in RAM but still links
# to debugging information for in-production debugging
RUN objcopy --only-keep-debug target/release/$BIN target/release/$BIN.dbg \
  && objcopy --strip-debug target/release/$BIN \
  && (cd target/release && objcopy --add-gnu-debuglink=$BIN.dbg $BIN)

###########################################################################
# Final Stage
###########################################################################
FROM debian:stable-slim as final
ARG BIN=sdf

RUN useradd -ms /bin/bash app \
  && mkdir -p {/run,/etc,/usr/local/etc,/home/app/.confg}/$BIN
WORKDIR /run/$BIN
COPY --from=builder /usr/src/target/release/$BIN /usr/local/bin/
COPY --from=builder /usr/src/target/release/$BIN.dbg /usr/local/bin/.debug/

EXPOSE 5156/tcp
ENTRYPOINT [ \
  "/sbin/runuser", "-u", "app", "--", "/usr/local/bin/sdf" \
]
