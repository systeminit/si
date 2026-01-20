# syntax=docker/dockerfile:1
# check=skip=InvalidDefaultArgInFrom

ARG IMAGE
ARG DIGEST

FROM ${IMAGE}@${DIGEST}

COPY ./rebaser /usr/local/bin/rebaser

ENTRYPOINT ["/usr/local/bin/rebaser"]
