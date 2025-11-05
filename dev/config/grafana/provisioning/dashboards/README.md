# Grafana Dashboard Guidelines

This directory contains Grafana dashboard definitions provisioned automatically to local development Grafana instances.

## Creating Portable Dashboards

All dashboards in this directory MUST be portable across environments without requiring manual modifications. This is accomplished using data source variables instead of hardcoded data source UIDs.

### Required Pattern: Data Source Variables

**✅ CORRECT - Use data source variables:**

Every dashboard must include a data source variable in the `templating` section:

```json
{
  "templating": {
    "list": [
      {
        "name": "datasource",
        "type": "datasource",
        "label": "Data Source",
        "query": "prometheus",
        "refresh": 1,
        "hide": 0
      }
    ]
  }
}
```

**Configuration Options:**

- `query`: Filter by data source type (`"prometheus"`, `"influxdb"`, etc.)
- `hide`:
  - `0` = visible dropdown (recommended for multi-environment deployments)
  - `1` = show variable name only
  - `2` = hidden (use when only one data source exists)
- `refresh`:
  - `1` = on dashboard load
  - `2` = on time range change

### Referencing the Data Source Variable

All data source references throughout the dashboard must use the variable:

**In Query Variables:**

```json
{
  "name": "service",
  "type": "query",
  "datasource": {
    "type": "prometheus",
    "uid": "${datasource}"
  },
  "query": "label_values(metric_name, label)"
}
```

**In Panel Configurations:**

```json
{
  "type": "timeseries",
  "title": "My Panel",
  "datasource": {
    "type": "prometheus",
    "uid": "${datasource}"
  },
  "targets": [
    {
      "datasource": {
        "type": "prometheus",
        "uid": "${datasource}"
      },
      "expr": "rate(my_metric[5m])"
    }
  ]
}
```

### Anti-Pattern: Hardcoded UIDs

**❌ FORBIDDEN - Never hardcode data source UIDs:**

```json
"datasource": {
  "type": "prometheus",
  "uid": "PBFA97CFB590B2093"  // ❌ This locks the dashboard to one environment
}
```

Hardcoded UIDs require manual modifications when deploying between environments and create maintenance burden.

## Dashboard Development Workflow

### 1. Design in Grafana UI

1. Open your local Grafana instance
2. Create a new dashboard or modify existing
3. **First step:** Add a data source variable in Settings → Variables
4. Use `${datasource}` in all panels and queries

### 2. Export Dashboard JSON

1. Navigate to Dashboard Settings → JSON Model
2. Copy the complete JSON
3. Save to this directory with a descriptive filename

### 3. Clean Up Exported JSON

Before committing, ensure:

- `"id": null` (Grafana assigns IDs dynamically)
- `"uid"`: Set to a meaningful identifier (e.g., `"layer-cache-metrics"`)
- `"version": 0` (Grafana increments on each save)
- Data source variable exists and is properly configured
- All datasource references use `${datasource}` or the variable name you chose

### 4. Validate Locally

**Local Testing:**
```bash
# Verify dashboard loads without errors in local Grafana
# Check that data source dropdown appears and functions
# Confirm all panels display data correctly
```

## Variable Naming Conventions

- **`datasource`**: Standard name for the Prometheus data source selector
- **`service`**: Service instance filter (from `exported_job` label)
- **`cache`**: Cache name filter
- Use descriptive names that clearly indicate what the variable filters

## Best Practices

### Performance

- Set appropriate refresh intervals (5s, 10s, 30s) based on metric cardinality
- Use `rate()` for counter metrics, not `increase()` with short ranges
- Avoid overly broad regex filters on high-cardinality labels

### Usability

- Add clear labels to variables: `"label": "Data Source"`
- Use `includeAll` option for filters when viewing aggregate data is useful
- Order variables logically (data source first, then filters)
- Group related panels using rows

### Maintenance

- Add descriptive titles and tags to dashboards
- Use consistent legend formats: `{{exported_job}}/{{label_name}}`
- Document non-obvious queries in panel descriptions
- Keep dashboard JSON in version control (this directory)

## Troubleshooting

### "Data source not found" errors

**Cause:** Dashboard is using hardcoded UIDs instead of variables

**Solution:** Replace all `"uid": "SPECIFIC_UID"` with `"uid": "${datasource}"`

### Variable dropdown is empty

**Cause:** No data sources match the `query` filter

**Solution:**
- Verify Prometheus data sources exist in the Grafana instance
- Check the variable's `query` field matches the data source type

### Queries fail after switching data sources

**Cause:** Metric names or labels differ between environments

**Solution:**
- Use label queries that work across all environments
- Consider adding metric name variables if schema differs
- Document environment-specific requirements

## Migration from Hardcoded UIDs

If you have existing dashboards with hardcoded data source UIDs:

1. Add a datasource variable to the `templating.list` array (see "Required Pattern" section above)
2. Replace all instances of `"uid": "HARDCODED_UID"` with `"uid": "${datasource}"`
3. Validate the JSON is still valid: `jq . < dashboard.json > /dev/null`
4. Test thoroughly in local Grafana before committing
5. Keep original in git history for rollback if needed

## References

- [Grafana Template Variables Documentation](https://grafana.com/docs/grafana/latest/dashboards/variables/)
- [Grafana Dashboard Best Practices](https://grafana.com/docs/grafana/latest/dashboards/build-dashboards/best-practices/)
- [Data Source Variables](https://grafana.com/docs/grafana/latest/dashboards/variables/add-template-variables/)
