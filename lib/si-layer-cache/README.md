# SI Layer Cache

Fast in-memory, network-aware, layered write-through cache for System Initiative.

## Features

- **Multi-layer caching:** Foyer (in-memory + disk) → S3 → PostgreSQL (legacy)
- **Network awareness:** NATS-based cache coherency across instances
- **Durable S3 writes:** Persistent queue with adaptive rate limiting
- **Observability:** Comprehensive metrics and structured logging

## S3 Write Queue

All S3 writes go through a persistent disk queue for durability:

- **No fast path:** Guarantees no data loss even on crash
- **ULID ordering:** Maintains chronological write order
- **Adaptive rate limiting:** Exponential backoff prevents constant throttling
- **Dead letter queue:** Preserves corrupted data for debugging

### Configuration

```json
{
  "object_storage": {
    "bucket": "my-cache-bucket",
    "key_prefix": "cache/",
    "rate_limit": {
      "max_delay_ms": 5000,
      "initial_backoff_ms": 100,
      "backoff_multiplier": 2.0,
      "successes_before_reduction": 3
    }
  }
}
```

See [config-example.json](./config-example.json) for full example.

### Monitoring

Key metrics for Grafana dashboards:

- `s3_write_queue_depth` - Pending writes (alert on sustained growth)
- `s3_write_backoff_ms` - Current backoff state (alert on sustained high values)
- `s3_write_attempts_total` - Success/throttle/error rates
- `s3_write_duration_ms` - End-to-end write latency

### Operational Notes

- **Queue location:** `{base_path}/{cache_name}_s3_queue/`
- **DLQ location:** `{base_path}/{cache_name}_s3_queue/dead_letter/`
- **Startup:** Automatically processes any pending queue entries
- **Shutdown:** Completes in-flight write, queue persists for restart
- **Rate limiter state:** Ephemeral, rediscovered naturally on startup

## Development

### Running Tests

```bash
# Unit tests
cargo test --package si-layer-cache

# Integration tests (requires S3 mock)
docker run -p 4566:4566 localstack/localstack
cargo test --package si-layer-cache --test s3_queue_integration_tests
```

### Documentation

- Module docs: `cargo doc --package si-layer-cache --open` - System architecture and component interaction
- [Config Schema](./config-schema.json) - JSON schema for configuration
