FROM alpine:latest AS setup

RUN apk add --no-cache wget tar yq && \
    wget -O /tmp/zed.tar.gz https://github.com/authzed/zed/releases/download/v0.21.5/zed_0.21.5_linux_amd64_gnu.tar.gz && \
    tar -xzf /tmp/zed.tar.gz -C /tmp && \
    chmod +x /tmp/zed

COPY ./schema.zed /tmp/schema.zed
COPY ./validation.yaml /tmp/validation.yaml

# stub the schema file into the validation.yaml
RUN sed -i '/<<SCHEMA>>/r /tmp/schema.zed' /tmp/validation.yaml && \
    sed -i '/<<SCHEMA>>/d' /tmp/validation.yaml

FROM authzed/spicedb:v1.37.0-debug

COPY --from=setup /tmp/zed /usr/local/bin/zed
COPY --from=setup /tmp/schema.zed .
COPY --from=setup /tmp/validation.yaml .
COPY ./entrypoint.sh .

ENTRYPOINT ["sh", "-c", "./entrypoint.sh"]
