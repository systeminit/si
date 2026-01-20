# syntax=docker/dockerfile:1
# check=skip=InvalidDefaultArgInFrom

ARG IMAGE
ARG DIGEST

FROM ${IMAGE}@${DIGEST}

COPY ./pinga /usr/local/bin/pinga

ENTRYPOINT ["/usr/local/bin/pinga"]
