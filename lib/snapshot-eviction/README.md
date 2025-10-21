# Snapshot Eviction

Garbage collection library for workspace snapshots that eliminates race conditions through recency-based tracking.

## Architecture

Zero generic type parameters. Uses PgLayer and LayeredEventClient from si-layer-cache directly.

```rust
pub struct SnapshotEvictor {
    si_db_pool: PgPool,                    // For queries + metadata
    layer_cache_pool: PgPool,              // For workspace_snapshots deletion
    layered_event_client: LayeredEventClient, // For NATS events
    config: SnapshotEvictionConfig,
}
```

## Integration with Forklift

Runs as tokio task spawned in forklift's try_run method:

```rust
let evictor = SnapshotEvictor::new(si_db_pool, layer_cache_pool, event_client, config);
tokio::spawn(async move {
    evictor.run(shutdown_token).await
});
```

## Configuration

```toml
[snapshot_eviction]
grace_period_seconds = 300    # 5 minutes (min: 30s)
poll_interval_seconds = 60    # 1 minute (min: 30s)
batch_size = 100

  [snapshot_eviction.si_db]
  dbname = "si"
  # ... PgPoolConfig fields

  [snapshot_eviction.layer_cache_pg]
  dbname = "si_layer_cache"
  # ... PgPoolConfig fields
```

## How It Works

### Race Condition Eliminated

**Problem:** Immediate eviction creates race where in-flight transactions fail when they commit.

**Solution:** Grace period (300s) >> transaction duration (<1s). By the time eviction check runs, any in-flight transactions have committed.

### Eviction Flow

1. **Query si-db** for snapshots unused for grace period with no references
2. **Delete from workspace_snapshots** (layer-cache DB) via PgLayer
3. **Publish NATS event** (Actor::System, Tenancy with empty ULIDs) for cache invalidation
4. **Delete metadata** from snapshot_last_used (si-db)

All operations autocommit. Failures logged but don't stop processing.

### Tracking Updates

When rebaser updates a change set pointer (via `ChangeSet::update_pointer`):

```sql
UPDATE change_set_pointers SET workspace_snapshot_address = $new WHERE id = $id;

INSERT INTO snapshot_last_used (snapshot_id, last_used_at, created_at)
VALUES ($old, CLOCK_TIMESTAMP(), CLOCK_TIMESTAMP())
ON CONFLICT (snapshot_id) DO UPDATE SET last_used_at = CLOCK_TIMESTAMP();
```

## Metrics

Published via `telemetry_utils` helper macros:

- `monotonic_counter.snapshot_eviction_cycles_completed` - Total eviction cycles completed
- `monotonic_counter.snapshot_eviction_successes` - Total successful evictions (recorded per-cycle as batch)
- `monotonic_counter.snapshot_eviction_failures` - Total failed evictions (recorded per-cycle as batch)
- `histogram.snapshot_eviction_candidates_found` - Distribution of candidates found per cycle (tracks workload characteristics)
- `histogram.snapshot_eviction_cycle_duration_seconds` - Complete cycle execution time in seconds
- `histogram.snapshot_eviction_duration_seconds` - Eviction duration distribution in seconds with `status` label
- `histogram.snapshot_candidate_queue_latency_seconds` - Time from eligibility to processing in seconds

Success and failure counters are emitted once per cycle with the batch totals, avoiding partial window artifacts in Prometheus queries.

Duration and queue latency are recorded in seconds to work with default OpenTelemetry histogram buckets (0-10000 range), providing good granularity for multi-second operations.

## Testing

```bash
cargo test -p snapshot-eviction
cargo test -p snapshot-eviction --test integration_test
```
