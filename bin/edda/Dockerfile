# syntax=docker/dockerfile:1
# check=skip=InvalidDefaultArgInFrom

ARG IMAGE
ARG DIGEST

FROM ${IMAGE}@${DIGEST}

COPY ./edda /usr/local/bin/edda

ENTRYPOINT ["/usr/local/bin/edda"]
