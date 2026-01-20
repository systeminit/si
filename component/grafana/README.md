# Grafana with Bundled Dashboards

This directory contains a custom Grafana Docker image that bundles System Initiative's monitoring dashboards and datasource configurations.

## Overview

The image is based on `grafana/grafana-enterprise` and includes all dashboards from `dev/config/grafana/provisioning/` baked directly into the image. When the container starts, the dashboards are immediately available without requiring volume mounts.

## Building the Image

### Using Docker Compose (in this directory):
```bash
docker-compose build
docker-compose up
```

### Using Docker directly:
```bash
docker build -t systeminit/grafana:local .
docker run -p 3000:3000 systeminit/grafana:local
```

### Using Buck2:
```bash
buck2 run //component/grafana:grafana
```

## Using with dev/docker-compose.platform.yml

The main development docker-compose file (`dev/docker-compose.platform.yml`) is configured to build this image automatically. Simply run:

```bash
cd dev
docker-compose -f docker-compose.platform.yml up grafana
```

## Updating Dashboards

To update the dashboards bundled in the image:

1. Make changes to the dashboard JSON files in `component/grafana/provisioning/dashboards/`
2. Rebuild the image using one of the methods above
3. Restart the container

Alternatively, you can sync changes from `dev/config/grafana/provisioning/`:

```bash
rsync -av --delete dev/config/grafana/provisioning/ component/grafana/provisioning/
```

## Configuration

### Environment Variables

The following environment variables are configured in the docker-compose file:

- `GF_AUTH_ANONYMOUS_ENABLED=true` - Enable anonymous access
- `GF_AUTH_ANONYMOUS_ORG_ROLE=Admin` - Give anonymous users admin role
- `GF_AUTH_DISABLE_LOGIN_FORM=true` - Disable login form

### Datasources

The image includes pre-configured datasources for:

- **Jaeger**: `http://dev-jaeger-1:16686` (traces)
- **Loki**: `http://loki:3100` (logs)
- **Prometheus**: `http://prometheus:9090` (metrics)

### Dashboards

All dashboard JSON files in `provisioning/dashboards/` are automatically loaded. See `provisioning/dashboards/README.md` for guidelines on creating portable dashboards.

## Accessing Grafana

Once running, access Grafana at: http://localhost:3000

No login is required in development mode.
