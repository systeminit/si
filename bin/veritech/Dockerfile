ARG BUILDER_IMAGE=cache
ARG RUST_VERSION=1.57.0

ARG BUILDER_LANG_JS_IMAGE=cache_lang_js
ARG NODE_VERSION=16.13.1

###########################################################################
# Cache Stage
###########################################################################
FROM rust:$RUST_VERSION as cache
ARG RUST_VERSION
ARG BIN=veritech

RUN apt-get update && apt-get install -y lld
WORKDIR /usr/src
COPY . .

RUN cargo build --release --bin $BIN --bin cyclone
RUN find . -maxdepth 1 ! -path './target' -and ! -path '.' -exec rm -rf {} \;

###########################################################################
# Builder Stage
###########################################################################
FROM $BUILDER_IMAGE as builder
ARG BIN=veritech

WORKDIR /usr/src
COPY . .

RUN cargo build --release --bin $BIN --bin cyclone
# Create a 2-part executable which occupies less space in RAM but still links
# to debugging information for in-production debugging
RUN objcopy --only-keep-debug target/release/$BIN target/release/$BIN.dbg \
  && objcopy --strip-debug target/release/$BIN \
  && (cd target/release && objcopy --add-gnu-debuglink=$BIN.dbg $BIN)

RUN objcopy --only-keep-debug target/release/cyclone target/release/cyclone.dbg \
  && objcopy --strip-debug target/release/cyclone \
  && (cd target/release && objcopy --add-gnu-debuglink=cyclone.dbg cyclone)

###########################################################################
# lang-js Cache Stage
###########################################################################
FROM node:$NODE_VERSION as cache_lang_js
ARG NODE_VERSION

WORKDIR /usr/src
COPY bin/lang-js/package.json bin/lang-js/package-lock.json .

RUN npm ci
RUN find . -maxdepth 1 ! -path './node_modules' -and ! -path '.' -exec rm -rf {} \;

###########################################################################
# lang-js Builder Stage
###########################################################################
FROM $BUILDER_LANG_JS_IMAGE as builder_lang_js

WORKDIR /usr/src
COPY bin/lang-js .

RUN env && npm run package && ls -la target

###########################################################################
# Final Stage
###########################################################################
FROM debian:stable-slim as final
ARG BIN=veritech

RUN useradd -ms /bin/bash app \
  && mkdir -p {/run,/etc,/usr/local/etc,/home/app/.confg}/$BIN
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
