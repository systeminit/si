FROM amazon/aws-cli:2.16.9

RUN set -eux; \
    yum update; \
    arch=$(arch | sed s/aarch64/arm64/ | sed s/x86_64/64bit/) && yum install -y \
        https://s3.amazonaws.com/session-manager-downloads/plugin/latest/linux_${arch}/session-manager-plugin.rpm;

COPY ./scripts/* /usr/local/bin/si/
ENV PATH="/usr/local/bin/si:${PATH}"

ENTRYPOINT ["bash", "-c"]
