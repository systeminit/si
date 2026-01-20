# syntax=docker/dockerfile:1
# check=skip=InvalidDefaultArgInFrom

ARG IMAGE
ARG DIGEST

FROM ${IMAGE}@${DIGEST}

COPY ./forklift /usr/local/bin/forklift

ENTRYPOINT ["/usr/local/bin/forklift"]
