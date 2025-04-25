ARG ALPINE_VERSION
FROM alpine:${ALPINE_VERSION}
WORKDIR /app
RUN apk --no-cache add aws-cli jq less curl
COPY ./configs /config
COPY ./docker-entrypoint.sh /app/
RUN chmod +x /app/docker-entrypoint.sh
ENTRYPOINT ["/app/docker-entrypoint.sh"]
