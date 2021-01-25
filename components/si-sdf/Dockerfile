FROM systeminit/si-builder:latest as build
LABEL maintainer="dev@systeminit.com"
RUN mkdir /src
WORKDIR /src
COPY . /src
RUN make build//components/si-sdf

FROM ubuntu:rolling
LABEL maintainer="dev@systeminit.com"
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y libssl1.1 libev-dev libevent-dev && \
  apt-get clean && \
  find /var/lib/apt/lists -type f | xargs rm && \
  find /var/log -type f -exec rm {} \; && \
  rm -rf /usr/share/man/* && \
  rm -rf /usr/share/doc/* && \
  rm -f /var/log/alternatives.log /var/log/apt/*
RUN mkdir -p /svc/si-sdf/config
COPY --from=build /src/target/debug/si-sdf-server /svc/si-sdf
COPY --from=build /src/components/si-sdf/config/* /svc/si-sdf/config
COPY --from=build /src/components/build/run.sh /svc
RUN chmod a+x /svc/run.sh
ENV START_SERVICE=si-sdf
EXPOSE 5156/tcp
ENTRYPOINT /svc/run.sh
