FROM alpine:3.18
WORKDIR /app
RUN apk --no-cache add aws-cli jq less
COPY ./configs /config
COPY ./docker-entrypoint.sh /app/
RUN chmod +x /app/docker-entrypoint.sh
ENTRYPOINT ["/app/docker-entrypoint.sh"]
