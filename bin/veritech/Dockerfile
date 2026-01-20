# syntax=docker/dockerfile:1
# check=skip=InvalidDefaultArgInFrom

ARG IMAGE
ARG DIGEST

FROM ${IMAGE}@${DIGEST}

COPY ./veritech /usr/local/bin/veritech

ENTRYPOINT ["/usr/local/bin/veritech"]
