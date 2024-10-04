FROM alpine:latest AS downloader

RUN apk add --no-cache wget tar
RUN wget -O /tmp/zed.tar.gz https://github.com/authzed/zed/releases/download/v0.21.5/zed_0.21.5_linux_amd64_gnu.tar.gz
RUN tar -xzf /tmp/zed.tar.gz -C /tmp
RUN chmod +x /tmp/zed

FROM authzed/spicedb:v1.37.0-debug

COPY --from=downloader /tmp/zed /usr/local/bin/zed
COPY ./schema.zed .
COPY ./entrypoint.sh .

ENTRYPOINT ["sh", "-c", "./entrypoint.sh"]
