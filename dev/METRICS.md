# Service Metrics Monitoring

This document describes how to monitor memory, CPU, and other system metrics for SI services during development and load testing.

## Overview

SI services (sdf, rebaser, pinga, veritech, cyclone) export process-level metrics via OpenTelemetry. The metrics flow through the following pipeline:

```text
Services → OpenTelemetry (OTLP) → otelcol → Prometheus → Grafana
```

## Quick Start

1. **Start the dev stack:**

   ```bash
   buck2 run dev:up
   ```

2. **Access Grafana:**
   - URL: <http://localhost:3000>
   - No authentication required (anonymous admin access enabled)

3. **View the dashboard:**
   - Navigate to "SI Service Metrics" dashboard
   - Dashboard automatically refreshes every 5 seconds

## Available Metrics

The following process-level metrics are collected for each service:

### Memory Metrics

- `process_runtime_memory_rss_bytes` - Resident Set Size (physical memory in bytes)
- `process_runtime_memory_virtual_bytes` - Virtual memory size (in bytes)

### CPU Metrics

- `process_runtime_cpu_usage_percent` - CPU usage percentage (can exceed 100% on multi-core machines)
- `process_runtime_cpu_time_milliseconds` - Accumulated CPU time in milliseconds (cumulative across all cores)

## Monitored Services

The dashboard tracks metrics for the following services:

- **sdf** - Main API server
- **rebaser** - Change set rebasing service
- **pinga** - Action execution service
- **veritech** - Function execution engine
- **cyclone** - Isolated workload runner

## Direct Access to Monitoring Tools

### Prometheus

- URL: <http://localhost:9090>
- Query metrics directly using PromQL
- Example query: `process_runtime_memory_rss_bytes{exported_job="sdf"}`

### OpenTelemetry Collector

- Metrics endpoint: <http://localhost:9090/metrics> (exported by otelcol)
- OTLP receiver (gRPC): localhost:4317
- OTLP receiver (HTTP): localhost:4318

### Jaeger (Distributed Tracing)

- URL: <http://localhost:16686>
- View distributed traces and request flows

### Loki (Logs)

- Query logs through Grafana's Loki datasource
- Logs are also available in `../log/` directory

## Creating Custom Dashboards

You can create custom Grafana dashboards by:

1. Navigate to Grafana (<http://localhost:3000>)
2. Click "+" → "Dashboard" → "Add visualization"
3. Select "Prometheus" as the datasource
4. Use PromQL to query metrics
5. Save the dashboard

Example queries:

```promql
# Memory usage for sdf service
process_runtime_memory_rss_bytes{exported_job="sdf"}

# Total memory across all services
sum(process_runtime_memory_rss_bytes)

# Virtual memory for all services
process_runtime_memory_virtual_bytes{exported_job=~"sdf|rebaser|pinga|veritech|cyclone"}
```

## Troubleshooting

### No metrics appearing in Grafana

1. **Check Prometheus is scraping otelcol:**
   - Visit <http://localhost:9090/targets>
   - Verify "otelcol-metrics" target is "UP"

2. **Verify services are exporting metrics:**
   - Visit <http://localhost:9090/metrics> (otelcol endpoint)
   - Search for `process_runtime_memory` metrics
   - You should see metrics with `exported_job` labels

3. **Check service logs:**

   ```bash
   # Check if process metrics were initialized
   docker logs dev-otelcol-1
   ```

### Dashboard shows "No data"

- Wait 10-15 seconds after starting services for first scrape
- Default scrape interval is 100ms (very fast)
- Check Prometheus datasource is configured correctly in Grafana

## Implementation Details

### Metrics Collection

Process metrics are collected using the `si-otel-metrics` crate, which is automatically initialized by the `telemetry-application` crate when services start. No manual initialization is required in service code.

### Export Configuration

Services export metrics via OTLP (OpenTelemetry Protocol):

- Endpoint: `otelcol:4317` (gRPC) or `otelcol:4318` (HTTP)
- Export interval: 1 second
- Batch size: Configured in telemetry-application-rs

### Prometheus Scrape Configuration

Prometheus scrapes metrics from otelcol's Prometheus exporter:

- Target: `otelcol:9090`
- Scrape interval: 100ms
- Scrape timeout: 90ms

Configuration: [dev/config/prometheus/config.yml](config/prometheus/config.yml)

## Additional Resources

- [OpenTelemetry Rust Documentation](https://opentelemetry.io/docs/languages/rust/)
- [Prometheus Query Documentation](https://prometheus.io/docs/prometheus/latest/querying/basics/)
- [Grafana Dashboard Documentation](https://grafana.com/docs/grafana/latest/dashboards/)
