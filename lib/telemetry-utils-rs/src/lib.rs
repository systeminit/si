/// This macro wraps tracing events with `metrics = true` to route them through the MetricsLayer.
/// The MetricsLayer recognizes four metric type prefixes based on the OpenTelemetry specification.
///
/// # Metric Types
///
/// ## `counter.` - Up/Down Counter
/// Values that can both increase and decrease. Use for tracking values that fluctuate in both directions.
///
/// **Use cases**: Active connections, queue depth, items in progress
///
/// **Example**:
/// ```rust
/// # use telemetry_utils::metric;
/// metric!(counter.connections.active = 1);   // Connection opened
/// metric!(counter.connections.active = -1);  // Connection closed
/// ```
///
/// ## `monotonic_counter.` - Monotonic Counter
/// Non-negative values that only increase over time. Use for cumulative totals.
///
/// **Use cases**: Total requests, bytes sent, errors occurred, events processed
///
/// **Example**:
/// ```rust
/// # use telemetry_utils::metric;
/// metric!(monotonic_counter.requests.total = 1);
/// metric!(monotonic_counter.bytes.sent = 1024);
/// ```
///
/// ## `histogram.` - Histogram
/// Distribution of values for statistical analysis. Records the distribution of measurements.
///
/// **Use cases**: Request duration, response size, query latency, processing time
///
/// **Example**:
/// ```rust
/// # use telemetry_utils::metric;
/// metric!(histogram.request.duration_ms = 125.5);
/// metric!(histogram.response.size_bytes = 4096);
/// ```
///
/// ## `gauge.` - Gauge
/// Instantaneous measurements that can arbitrarily go up or down. Represents a point-in-time value.
///
/// **Use cases**: Memory usage, CPU percentage, temperature, current queue size
///
/// **Example**:
/// ```rust
/// # use telemetry_utils::metric;
/// metric!(gauge.memory.usage_bytes = 1073741824);
/// metric!(gauge.cpu.percent = 75.5);
/// ```
///
/// # Adding Labels (Attributes)
///
/// Metrics support additional key-value labels for dimensionality:
///
/// ```rust
/// # use telemetry_utils::metric;
/// // Single label
/// metric!(counter.requests = 1, method = "GET");
///
/// // Multiple labels
/// metric!(histogram.api.latency_ms = 42.5, method = "POST", endpoint = "/users", status = 200);
///
/// // Dynamic labels
/// let service_name = "poop"
/// metric!(histogram.api.latency_ms = 42.5, method = "POST", service = service_name);
/// ```
///
/// # Important Constraints
///
/// - **Do NOT mix** floating-point and integer numbers for the same metric name
/// - Integers (i64/u64) can be mixed and will be handled automatically
///
/// ```
#[macro_export]
macro_rules! metric {
    ($key:ident = $value:expr_2021, *) => {
        info!(metrics = true, $key = $value, *);
    };
    ($key:literal = $value:expr_2021, *) => {
        info!(metrics = true, $key = $value, *);
    };
    ($($key:ident).+ = $value:expr_2021) => {
        info!(metrics = true, $($key).+ = $value);
    };
    ($($key:ident).+ = $value:expr_2021, $($label:ident = $label_value:expr_2021),+ $(,)?) => {
        info!(metrics = true, $($key).+ = $value, $($label = $label_value),+);
    };
}

/// Emit a counter metric (can increase or decrease).
///
/// Counters track changes in a value that can go both up and down. Use this for tracking
/// active connections, items in progress, or queue depth.
///
/// # Examples
///
/// ```rust
/// # use telemetry_utils::counter;
/// // Increment
/// counter!(connections.active = 1);
///
/// // Decrement
/// counter!(connections.active = -1);
///
/// // With labels
/// counter!(tasks.running = 1, worker_id = 42);
/// counter!(queue.depth = -1, queue_name = "events", priority = "high");
/// ```
#[macro_export]
macro_rules! counter {
    ($($key:ident).+ = $value:expr_2021 $(, $label:ident = $label_value:expr_2021)* $(,)?) => {
        $crate::metric!(counter.$($key).+ = $value $(, $label = $label_value)*);
    };
}

/// Emit a monotonic counter metric (only increases).
///
/// Monotonic counters track cumulative totals that only ever increase. Use this for
/// counting total requests, bytes sent, errors occurred, or events processed.
///
/// # Examples
///
/// ```rust
/// # use telemetry_utils::monotonic;
/// monotonic!(requests.total = 1);
/// monotonic!(bytes.sent = 1024);
/// monotonic!(errors.occurred = 1, error_type = "timeout");
/// ```
#[macro_export]
macro_rules! monotonic {
    ($($key:ident).+ = $value:expr_2021 $(, $label:ident = $label_value:expr_2021)* $(,)?) => {
        $crate::metric!(monotonic_counter.$($key).+ = $value $(, $label = $label_value)*);
    };
}

/// Emit a histogram metric (distribution of values).
///
/// Histograms record the statistical distribution of measurements. Use this for tracking
/// request durations, response sizes, query latencies, or any value where you want to
/// analyze percentiles, averages, or distributions.
///
/// # Examples
///
/// ```rust
/// # use telemetry_utils::histogram;
/// histogram!(request.duration_ms = 125.5);
/// histogram!(response.size_bytes = 4096);
/// histogram!(query.latency_ms = 23.4, query_type = "select", table = "users");
/// ```
#[macro_export]
macro_rules! histogram {
    ($($key:ident).+ = $value:expr_2021 $(, $label:ident = $label_value:expr_2021)* $(,)?) => {
        $crate::metric!(histogram.$($key).+ = $value $(, $label = $label_value)*);
    };
}

/// Emit a gauge metric (instantaneous measurement).
///
/// Gauges represent point-in-time measurements that can arbitrarily go up or down.
/// Use this for reporting current memory usage, CPU percentage, temperature, or
/// instantaneous queue sizes.
///
/// Note: Gauges differ from counters in that they represent absolute values rather than
/// changes. Set a gauge to the current value, don't increment/decrement it.
///
/// # Examples
///
/// ```rust
/// # use telemetry_utils::gauge;
/// gauge!(memory.heap_bytes = 1073741824);
/// gauge!(cpu.percent = 75.5);
/// gauge!(temperature.celsius = 42.8, sensor_id = "cpu-1");
/// gauge!(queue.current_size = 150, queue_name = "events");
/// ```
#[macro_export]
macro_rules! gauge {
    ($($key:ident).+ = $value:expr_2021 $(, $label:ident = $label_value:expr_2021)* $(,)?) => {
        $crate::metric!(gauge.$($key).+ = $value $(, $label = $label_value)*);
    };
}
