FROM bitnami/minideb:latest

RUN install_packages curl ca-certificates
RUN curl -sSfL https://auth.systeminit.com/install.sh | sh

ENTRYPOINT [ "/usr/local/bin/si" ]