ARG ALPINE_VERSION
FROM alpine:${ALPINE_VERSION} AS setup

COPY ./schema.zed /tmp/schema.zed
COPY ./validation.yaml /tmp/validation.yaml

# stub the schema file into the validation.yaml
RUN sed -i '/<<SCHEMA>>/r /tmp/schema.zed' /tmp/validation.yaml && \
    sed -i '/<<SCHEMA>>/d' /tmp/validation.yaml

ARG AUTHZED_ZED_VERSION
FROM authzed/zed:${AUTHZED_ZED_VERSION} AS cli

ARG AUTHZED_SPICEDB_VERSION
FROM authzed/spicedb:${AUTHZED_SPICEDB_VERSION}

COPY --from=cli /usr/local/bin/zed /usr/local/bin/zed
COPY --from=setup /tmp/schema.zed .
COPY --from=setup /tmp/validation.yaml .
COPY --chmod=755 ./entrypoint.sh .

ENTRYPOINT ["sh", "-c", "./entrypoint.sh"]
