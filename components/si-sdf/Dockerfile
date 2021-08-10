ARG BUILDER_IMAGE=cache
ARG RUST_VERSION=1.54.0

###########################################################################
# Cache Stage
###########################################################################
FROM rust:$RUST_VERSION as cache
ARG RUST_VERSION
ARG BIN=si-sdf-server
ARG SVC=si-sdf

RUN apt-get update && apt-get install -y lld
WORKDIR /usr/src
COPY . .

RUN cargo build --release --bin $BIN
RUN find . -maxdepth 1 ! -path './target' -and ! -path '.' -exec rm -rf {} \;

###########################################################################
# Builder Stage
###########################################################################
FROM $BUILDER_IMAGE as builder
ARG BIN=si-sdf-server

WORKDIR /usr/src
COPY . .

RUN cargo build --release --bin $BIN
# Create a 2-part executable which occupies less space in RAM but still links
# to debugging information for in-production debugging
RUN objcopy --only-keep-debug target/release/$BIN target/release/$BIN.dbg \
  && objcopy --strip-debug target/release/$BIN \
  && (cd target/release && objcopy --add-gnu-debuglink=$BIN.dbg $BIN)

###########################################################################
# Dockerize Stage
###########################################################################
FROM debian:stable-slim as dockerize
ARG DOCKERIZE_VERSION=v0.6.1

RUN apt-get update && apt-get install -y wget
RUN wget https://github.com/jwilder/dockerize/releases/download/$DOCKERIZE_VERSION/dockerize-linux-amd64-$DOCKERIZE_VERSION.tar.gz \
    && tar -C /usr/local/bin -xzvf \
      dockerize-linux-amd64-$DOCKERIZE_VERSION.tar.gz \
    && rm dockerize-linux-amd64-$DOCKERIZE_VERSION.tar.gz

###########################################################################
# Final Stage
###########################################################################
FROM debian:stable-slim as final
ARG BIN=si-sdf-server

RUN useradd -ms /bin/bash app
WORKDIR /svc/si-sdf
COPY --from=dockerize /usr/local/bin/dockerize /usr/local/bin/dockerize
COPY --from=builder /usr/src/target/release/si-sdf-server ./
COPY --from=builder /usr/src/target/release/si-sdf-server.dbg ./.debug/
COPY --from=builder /usr/src/components/si-sdf/config/* ./config/

EXPOSE 5156/tcp
ENTRYPOINT [ \
  "/sbin/runuser", "-u", "app", "--", \
    "/usr/local/bin/dockerize", \
      "-timeout", "30s", \
      "-wait", "tcp://postgres:5432", \
      "-wait", "tcp://nats:4222", \
      "-wait", "tcp://otelcol:4317", \
      "-wait", "tcp://veritech:5157", \
      "./si-sdf-server" \
]
