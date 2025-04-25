ARG ALPINE_VERSION
FROM alpine:${ALPINE_VERSION}

# Set the working directory
WORKDIR /app

# Install required packages: aws-cli, jq, less, curl, python3, pip, and required Python packages
RUN apk --no-cache add aws-cli jq less curl python3 py3-pip && \
    pip3 install boto3 botocore requests

# Copy your Python functions into the /functions directory
COPY ./functions /functions

# Copy the entrypoint script
COPY ./docker-entrypoint.sh /app/

# Make the entrypoint script executable
RUN chmod +x /app/docker-entrypoint.sh

# Set the entrypoint
ENTRYPOINT ["/app/docker-entrypoint.sh"]