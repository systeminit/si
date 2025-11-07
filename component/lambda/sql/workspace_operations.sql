CREATE SCHEMA IF NOT EXISTS workspace_operations;

GRANT USAGE ON SCHEMA workspace_operations TO lambda_user;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA workspace_operations TO lambda_user;

-- Used to track upload progress for various billing uploads
CREATE TABLE IF NOT EXISTS workspace_operations.upload_progress (
    -- The type of upload (e.g. "posthog-workspace_resource_hours")
    upload_type TEXT PRIMARY KEY,
    -- The time at which the upload stops (*exclusive*)
    uploaded_to TIMESTAMP NOT NULL
)

-- Parse an ISO-8601 timestamp in format 2025-04-03T22:19:42.999999945Z.
-- SQL ::timestamp has a bug and can't parse .999999500-.999999999, so we truncate the last 3 digits.
-- (We also truncate the Z, but Redshift still parses it as UTC)
CREATE OR REPLACE FUNCTION workspace_operations.parse_timestamp(TEXT)
    RETURNS TIMESTAMP
    STABLE
    AS $$
        SELECT SUBSTRING($1, 1, 26)::timestamp
    $$ LANGUAGE SQL;

-- Imported mapping from a workspace to its owner.
-- This table creation is untested (it was created manually and this description extracted)
CREATE TABLE IF NOT EXISTS workspace_operations.workspace_owners (
    owner_pk VARCHAR(255),
    workspace_id VARCHAR(255),
    record_timestamp TIMESTAMP NOT NULL DEFAULT now(),
    PRIMARY KEY (owner_pk, workspace_id)
);

-- Generates a list of all hours from Sep. 2024 until now
CREATE OR REPLACE VIEW workspace_operations.si_hours AS
    WITH
        RECURSIVE hour_generator (hour_start) AS (
            SELECT workspace_operations.parse_timestamp('2024-09-01T00:00:00.000000000Z')
            UNION ALL (
                SELECT hour_start + INTERVAL '1 hour' HOUR
                FROM hour_generator
                WHERE hour_start < getdate()
            )
        )
    SELECT hour_start FROM hour_generator;

-- List of all owners (one row per owner)
CREATE OR REPLACE VIEW workspace_operations.owners AS
    SELECT
        owner_pk,
        MIN(record_timestamp) AS first_timestamp,
        DATE_TRUNC('hour', first_timestamp) AS first_hour,
        COUNT(DISTINCT workspace_id) AS workspace_count
      FROM workspace_operations.workspace_owners GROUP BY owner_pk;

-- List of all workspaces (one row per workspace)
CREATE OR REPLACE VIEW workspace_operations.workspaces AS
    SELECT
        workspace_id,
        MIN(record_timestamp) AS first_timestamp,
        DATE_TRUNC('hour', first_timestamp) AS first_hour,
        COUNT(DISTINCT owner_pk) AS owner_count
      FROM workspace_operations.workspace_owners GROUP BY workspace_id;

-- One row per owner per hour from Sep. 2024 until now (excluding hours where an owner had no workspace)
CREATE OR REPLACE VIEW workspace_operations.workspace_si_hours AS
    SELECT *
      FROM workspace_operations.si_hours
     CROSS JOIN workspace_operations.workspaces
     WHERE first_hour <= hour_start;

-- One row per workspace per hour from the workspace start time until now)
CREATE OR REPLACE VIEW workspace_operations.owner_si_hours AS
    SELECT *
      FROM workspace_operations.si_hours
     CROSS JOIN workspace_operations.owners
     WHERE first_hour <= hour_start;

-- workspace_update_events, materialized, with data cleanup.
--
-- NOTE: this does not auto refresh (and cannot, since it's pulling from an external schema).
-- We have a scheduled "REFRESH MATERIALIZED VIEW" query set up in Redshift. If you recreate
-- this view or move to a new Redshift, you will 
--
-- NOTE: this also does not *incrementally* refresh, also because it's an external schema; each
-- refresh rebuilds the whole view. This is not intended as a permanent situation; the pipeline
-- is due for a rebuild, but we want to wait a bit as our requirements are likely to change as
-- well in the near future.
CREATE MATERIALIZED VIEW workspace_operations.workspace_update_events_clean
  DISTKEY (workspace_id)
  SORTKEY (workspace_id, event_timestamp)
  AS
    SELECT workspace_id,
           change_set_id,
           workspace_operations.parse_timestamp(event_timestamp) AS event_timestamp,
           workspace_snapshot_address,
           change_set_status,
           merge_requested_by_user_id,
           resource_count,
           component_id,
           component_name,
           schema_variant_id,
           schema_id,
           schema_name,
           func_run_id,
           kind,
           partition_0,
           partition_1,
           partition_2,
           partition_3
      -- This is the ONLY QUERY that should read from this external schema. Use
      -- this materialized view instead.
      FROM workspace_update_events.workspace_update_events;

-- Status of workspace_update_events ingestion, including when we consider events to be complete
CREATE OR REPLACE VIEW workspace_operations.workspace_update_events_summary AS
    SELECT
        workspace_operations.parse_timestamp('2024-09-01T00:00:00.000000000Z') AS launch_start,
        MIN(event_timestamp) AS first_event,
        MAX(event_timestamp) AS last_incomplete_event,
        -- We assume there may be some events that come in out-of-order within a 15-minute
        -- period, but that no more out-of-order events will come in from before that.
        -- NOTE: If there is no activity (or very little activity) neither of these assumptions hold.
        last_incomplete_event - (INTERVAL '15 minutes' MINUTE) AS last_complete_event,
        -- The last complete *hour* is the hour before that 15-minute cutoff (if there is an
        -- hour with some complete events and some incomplete events, that hour is incomplete).
        DATE_TRUNC('hour', last_complete_event) AS last_complete_hour_end,
        last_complete_hour_end - (INTERVAL '1 hour' HOUR) AS last_complete_hour_start
      FROM workspace_operations.workspace_update_events_clean
WITH NO SCHEMA BINDING;

-- The resource counts for each workspace after each time it changes
CREATE OR REPLACE VIEW workspace_operations.workspace_resource_counts AS
    WITH
        -- Get only events with actual resource_counts recorded (plus previous resource count for next step)
        workspace_resource_count_events AS (
            SELECT *, LAG(resource_count) OVER (PARTITION BY workspace_id ORDER BY event_timestamp) AS prev_resource_count
            FROM workspace_operations.workspace_update_events_clean
            WHERE resource_count IS NOT NULL
        )
    -- Once we have only events with resource counts, get the ones where it changed (plus next_event_timestamp)
    SELECT workspace_id,
           event_timestamp,
           resource_count,
           prev_resource_count,
           LEAD(event_timestamp) OVER (PARTITION BY workspace_id ORDER BY event_timestamp) AS next_event_timestamp
      FROM workspace_resource_count_events
     WHERE resource_count != prev_resource_count OR prev_resource_count IS NULL
WITH NO SCHEMA BINDING;

-- Get the resource counts for all workspaces anytime they change *and* at least once per hour
CREATE OR REPLACE VIEW workspace_operations.workspace_hourly_samples AS
    WITH
        -- Get all times at which any workspace for an owner has changed
        workspace_change_events AS (
            SELECT owner_pk, event_timestamp
            FROM workspace_operations.workspace_resource_counts
            JOIN workspace_operations.workspace_owners USING (workspace_id)
        ),
        -- Get times at which we want to sample workspace resource counts
        --
        -- Using UNION instead of UNION ALL removes duplicates introduced when two
        -- workspaces change at the same time, as well as when a workspace changes at
        -- the exact start of an hour
        sample_times (owner_pk, hour_start, sample_time) AS (
            (SELECT owner_pk, DATE_TRUNC('hour', event_timestamp), event_timestamp FROM workspace_change_events)
            UNION
            (SELECT owner_pk, hour_start, hour_start FROM workspace_operations.owner_si_hours)
        )
    SELECT owner_pk,
           hour_start,
           sample_time,
           workspace_id,
           event_timestamp,
           resource_count,
           -- prev_resource_count is only different if this event happened at the sample time
           CASE WHEN sample_time = event_timestamp THEN prev_resource_count ELSE resource_count END AS prev_resource_count
      FROM sample_times
      JOIN workspace_operations.workspace_owners USING (owner_pk)
      JOIN workspace_operations.workspace_resource_counts USING (workspace_id)
     WHERE sample_time >= event_timestamp
       AND (next_event_timestamp IS NULL OR sample_time < next_event_timestamp)
WITH NO SCHEMA BINDING;

-- Get the maximum resource count for each owner at each hour (and the time at which it occurred)
CREATE OR REPLACE VIEW workspace_operations.owner_resource_hours AS
WITH
    owner_samples AS (
        SELECT owner_pk,
               hour_start,
               sample_time,
               SUM(resource_count) AS owner_resource_count,
               SUM(prev_resource_count) AS owner_prev_resource_count,
               -- We want to pick the largest resource count for each owner at each hour
               ROW_NUMBER() OVER (PARTITION BY owner_pk, hour_start ORDER BY owner_resource_count DESC, sample_time) AS resource_count_order
          FROM workspace_operations.workspace_hourly_samples
         GROUP BY owner_pk, hour_start, sample_time
    )
    SELECT owner_pk,
           hour_start,
           sample_time,
           owner_resource_count AS resource_count,
           owner_prev_resource_count AS prev_resource_count
      FROM owner_samples
     WHERE resource_count_order = 1
WITH NO SCHEMA BINDING;

-- Get the resource counts for all workspaces at the point of the maximum resource count of
-- their owner (owner_resource_hours)
CREATE OR REPLACE VIEW workspace_operations.workspace_resource_hours AS
    SELECT workspace_hourly_samples.*
      FROM workspace_operations.owner_resource_hours
      JOIN workspace_operations.workspace_hourly_samples USING (owner_pk, sample_time)
WITH NO SCHEMA BINDING;

-- Totals and max count for the month
CREATE OR REPLACE VIEW workspace_operations.owner_billing_months AS
    SELECT
        owner_pk,
        DATE_TRUNC('month', hour_start) AS month,
        MAX(resource_count) AS max_resource_count,
        SUM(resource_count) AS resource_hours,
        (MAX(resource_count) <= 100) AS is_free
    FROM workspace_operations.owner_resource_hours
    GROUP BY owner_pk, DATE_TRUNC('month', hour_start)
WITH NO SCHEMA BINDING;


--
-- Lago subscriptions
--

-- Subscriptions for each owner, imported from Lago
-- This table creation is untested (it was created manually and this description extracted)
CREATE TABLE IF NOT EXISTS workspace_operations.workspace_owner_subscriptions (
    owner_pk VARCHAR(255),
    subscription_id VARCHAR(255),
    subscription_start_date TIMESTAMP,
    subscription_end_date TIMESTAMP,
    plan_code character VARCHAR(255),
    record_timestamp TIMESTAMP NOT NULL DEFAULT now(),
    external_id character VARCHAR(255),
    PRIMARY KEY (owner_pk, subscription_id)
);

-- Get the latest subscription records for each owner (the ones with MAX(record_timestamp))
-- with extra data 
CREATE OR REPLACE VIEW workspace_operations.latest_owner_subscriptions AS
    WITH
      -- Find the latest time we've downloaded subscription data for each owner. That timestamp identifies the batch
      latest_owner_subscription_imports AS (
        SELECT owner_pk, MAX(record_timestamp) AS record_timestamp
          FROM workspace_operations.workspace_owner_subscriptions
         GROUP BY owner_pk
      )
    -- Link subscription records in order: the next subscription the user has (as long as it hasn't happened yet) will
    -- have a NULL start and end date, and its start date is really the end date of the previous subscription.
    SELECT
        *,
        -- Subscription starts at the beginning of the day
        DATE_TRUNC('day', subscription_start_date) AS subscription_start_time,
        -- The end date is actually set to the *start* of the last day of the subscription; we need to cover all
        -- the hours *during* that day under the subscription, so we add a day (end_time is exclusive so it
        -- won't include the first hour of the next day)
        DATEADD(DAY, 1, DATE_TRUNC('day', subscription_end_date)) AS subscription_end_time,
        -- Get the previous subscription in order so we can seal up holes in the date
        LAG(subscription_end_time) OVER (PARTITION BY owner_pk ORDER BY subscription_start_date) AS prev_subscription_end_time,
        -- Get the next subscription in order (used by workspace_verifications)
        LEAD(subscription_id) OVER (PARTITION BY owner_pk ORDER BY subscription_start_date) AS next_subscription_id,
        LEAD(subscription_start_time) OVER (PARTITION BY owner_pk ORDER BY subscription_start_time) AS next_subscription_start_time,
        -- We start when the previous subscription ends, if no start time was set.
        COALESCE(subscription_start_time, prev_subscription_end_time) AS start_time,
        -- We end when the next subscription starts, unless there is a gap (which will be detected elsewhere).
        -- If there is no next subscription start, we use our end time, for whatever that's worth
        COALESCE(next_subscription_start_time, subscription_end_time) AS end_time
    FROM latest_owner_subscription_imports
    JOIN workspace_operations.workspace_owner_subscriptions USING (owner_pk, record_timestamp);

-- Associate each hour with its subscription for each owner.
CREATE OR REPLACE VIEW workspace_operations.owner_si_hours_with_subscriptions AS
    SELECT *, latest_owner_subscriptions.owner_pk IS NOT NULL AS subscription_exists
      FROM workspace_operations.owner_si_hours
      LEFT OUTER JOIN workspace_operations.latest_owner_subscriptions USING (owner_pk)
     WHERE latest_owner_subscriptions.owner_pk IS NULL
        OR (hour_start >= start_time AND (end_time IS NULL OR hour_start < end_time));

-- Get owner_resource_hours with subscription data
CREATE OR REPLACE VIEW workspace_operations.owner_resource_hours_with_subscriptions AS
    SELECT owner_resource_hours.*, owner_si_hours_with_subscriptions.external_id AS external_subscription_id, owner_si_hours_with_subscriptions.plan_code
      FROM workspace_operations.owner_resource_hours
      LEFT OUTER JOIN workspace_operations.owner_si_hours_with_subscriptions USING (owner_pk, hour_start)
        -- Include data after launch even if it doesn't have a subscription so we can find issues
     WHERE hour_start >= '2024-09-25'::timestamp OR subscription_exists
    WITH NO SCHEMA BINDING;
