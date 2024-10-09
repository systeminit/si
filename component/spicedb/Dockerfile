FROM alpine:latest AS setup

COPY ./schema.zed /tmp/schema.zed
COPY ./validation.yaml /tmp/validation.yaml

# stub the schema file into the validation.yaml
RUN sed -i '/<<SCHEMA>>/r /tmp/schema.zed' /tmp/validation.yaml && \
    sed -i '/<<SCHEMA>>/d' /tmp/validation.yaml

FROM authzed/zed:v0.21.5 AS cli
FROM authzed/spicedb:v1.37.0-debug

COPY --from=cli /usr/local/bin/zed /usr/local/bin/zed
COPY --from=setup /tmp/schema.zed .
COPY --from=setup /tmp/validation.yaml .
COPY ./entrypoint.sh .

ENTRYPOINT ["sh", "-c", "./entrypoint.sh"]
