FROM systeminit/si-builder:latest as build
LABEL maintainer="dev@systeminit.com"
RUN mkdir /src
WORKDIR /src
COPY . /src
RUN make build//components/si-registry

FROM ubuntu:rolling
LABEL maintainer="dev@systeminit.com"
ARG NODE_VERSION=v14.10.1
ARG NODE_DISTRO=linux-x64
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y libssl1.1 openssl curl xz-utils libev-dev libevent-dev && \ 
  apt-get autoremove -y && \
  apt-get clean && \
  find /var/lib/apt/lists -type f | xargs rm && \
  find /var/log -type f -exec rm {} \; && \
  rm -rf /usr/share/man/* && \
  rm -rf /usr/share/doc/* && \
  rm -f /var/log/alternatives.log /var/log/apt/* 
RUN curl -sSf https://nodejs.org/dist/$NODE_VERSION/node-$NODE_VERSION-$NODE_DISTRO.tar.xz -o /tmp/node-$NODE_VERSION-$NODE_DISTRO.tar.xz && \
  mkdir -p /usr/local/lib/nodejs && \
  tar -xJvf /tmp/node-$NODE_VERSION-$NODE_DISTRO.tar.xz -C /usr/local/lib/nodejs && \
  ln -sf /usr/local/lib/nodejs/node-$NODE_VERSION-$NODE_DISTRO/bin/node /usr/bin/node && \
  ln -sf /usr/local/lib/nodejs/node-$NODE_VERSION-$NODE_DISTRO/bin/npm /usr/bin/npm && \
  ln -sf /usr/local/lib/nodejs/node-$NODE_VERSION-$NODE_DISTRO/bin/npx /usr/bin/npx && \
  rm /tmp/node-$NODE_VERSION-$NODE_DISTRO.tar.xz 
RUN mkdir -p /svc/si-veritech
COPY --from=build /src/components/si-registry/dist/server.bundle.js /svc/si-veritech
COPY --from=build /src/components/si-registry/node_modules /svc/si-veritech/node_modules
COPY --from=build /src/components/build/run.sh /svc
RUN chmod a+x /svc/run.sh
ENV START_SERVICE=si-veritech
EXPOSE 5157/tcp
ENTRYPOINT /svc/run.sh
