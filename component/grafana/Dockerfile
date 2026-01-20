ARG BASE_VERSION=latest
FROM grafana/grafana-enterprise:${BASE_VERSION}

# Copy provisioning configuration and dashboards into the image
COPY provisioning/datasources /etc/grafana/provisioning/datasources
COPY provisioning/dashboards /etc/grafana/provisioning/dashboards

# Ensure proper ownership (grafana runs as user 472)
USER root
RUN chown -R 472:472 /etc/grafana/provisioning
USER 472
