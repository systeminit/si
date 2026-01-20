# syntax=docker/dockerfile:1
# check=skip=InvalidDefaultArgInFrom

ARG IMAGE
ARG DIGEST

FROM ${IMAGE}@${DIGEST}

COPY ./sdf /usr/local/bin/sdf

ENTRYPOINT ["/usr/local/bin/sdf"]
