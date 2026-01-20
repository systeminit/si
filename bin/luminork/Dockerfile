# syntax=docker/dockerfile:1
# check=skip=InvalidDefaultArgInFrom

ARG IMAGE
ARG DIGEST

FROM ${IMAGE}@${DIGEST}

COPY ./luminork /usr/local/bin/luminork

ENTRYPOINT ["/usr/local/bin/luminork"]
