FROM alpine:3

RUN apk add --no-cache \
    curl \
    aws-cli \
    bash

WORKDIR /app

COPY health-monitor.sh /app/health-monitor.sh
RUN chmod +x /app/health-monitor.sh

ENV HEALTH_ENDPOINT=http://127.0.0.1:8080/health
ENV SLEEP_INTERVAL=30
ENV AWS_REGION=us-east-1

ENTRYPOINT ["/app/health-monitor.sh"]