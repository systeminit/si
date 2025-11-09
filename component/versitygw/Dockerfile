ARG BASE_VERSION=latest

FROM versity/versitygw:${BASE_VERSION}
ARG BASE_VERSION

# Install utilities for init script
RUN apk add --no-cache bash

# Copy init script to create buckets on startup
COPY init-buckets.sh /usr/local/bin/init-buckets.sh
RUN chmod +x /usr/local/bin/init-buckets.sh

# Note: init-buckets.sh will be called by docker-compose command
