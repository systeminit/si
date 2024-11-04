CREATE SCHEMA IF NOT EXISTS workspace_operations;

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
            SELECT '2024-09-01 00:00:00'::timestamp
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
     CROSS JOIN workspace_operations.workspaces;
--     WHERE first_hour <= hour_start; TODO reintroduce this; it was removed to ensure identical results between old and new, but these are unnecessary extra resource_hour records that will get backfilled

-- One row per owner per hour from Sep. 2024 until now (excluding hours where an owner had no workspace)
CREATE OR REPLACE VIEW workspace_operations.owner_si_hours AS
    SELECT *
      FROM workspace_operations.si_hours
     CROSS JOIN workspace_operations.owners;
--     WHERE first_hour <= hour_start; TODO reintroduce this; it was removed to ensure identical results between old and new, but these are unnecessary extra resource_hour records that will get backfilled

-- Status of workspace_update_events ingestion, including when we consider events to be complete
CREATE OR REPLACE VIEW workspace_operations.workspace_update_events_summary AS
    SELECT
        MIN(event_timestamp)::timestamp AS first_event,
        MAX(event_timestamp)::timestamp AS last_incomplete_event,
        last_incomplete_event - (INTERVAL '15 minutes' MINUTE) AS last_complete_event,
        DATE_TRUNC('hour', last_complete_event) AS last_complete_hour_end,
        last_complete_hour_end - (INTERVAL '1 hour' HOUR) AS last_complete_hour_start
      FROM workspace_update_events.workspace_update_events
WITH NO SCHEMA BINDING;


-- The resource counts for each workspace after each time it changes
CREATE OR REPLACE VIEW workspace_operations.workspace_resource_counts AS
    WITH
        -- Get only events with actual resource_counts recorded (plus previous resource count for next step)
        workspace_resource_count_events AS (
            SELECT *, LAG(resource_count) OVER (PARTITION BY workspace_id ORDER BY event_timestamp) AS prev_resource_count
            FROM workspace_update_events.workspace_update_events
            WHERE resource_count IS NOT NULL
        ),
        -- Once we have only events with resource counts, get the ones where it changed (plus next_event_timestamp)
        workspace_resource_count_change_events AS (
            SELECT *, LEAD(event_timestamp) OVER (PARTITION BY workspace_id ORDER BY event_timestamp) AS next_event_timestamp
              FROM workspace_resource_count_events
             WHERE resource_count != prev_resource_count OR prev_resource_count IS NULL
        )
    SELECT workspace_id, event_timestamp::timestamp, resource_count, next_event_timestamp::timestamp
      FROM workspace_resource_count_change_events
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
    SELECT owner_pk, hour_start, sample_time, workspace_id, event_timestamp, resource_count
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
               ROW_NUMBER() OVER (PARTITION BY owner_pk, hour_start ORDER BY owner_resource_count DESC) AS resource_count_order
          FROM workspace_operations.workspace_hourly_samples
         GROUP BY owner_pk, hour_start, sample_time
    )
    SELECT owner_pk, hour_start, sample_time, owner_resource_count AS resource_count
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
CREATE OR REPLACE VIEW workspace_operations.latest_owner_subscriptions AS
    WITH
      -- Find the latest time we've downloaded subscription data for each owner
      latest_subscription_record_timestamps AS (
        SELECT owner_pk, MAX(record_timestamp) AS latest_record_timestamp
          FROM workspace_operations.workspace_owner_subscriptions
         GROUP BY owner_pk
      ),
      -- If the user's subscription data has changed, we only want the latest (MAX(record_timestamp))
      latest_subscription_records AS (
        SELECT workspace_owner_subscriptions.*,
               -- Start dates on subscriptions need to include the whole day even though they sometimes have HH:MM:SS
               -- in Lago
               DATE_TRUNC('day', subscription_start_date) AS start_time,
               -- The end date is actually set to the *start* of the last day of the subscription; we need to cover all
               -- the hours *during* that day under the subscription, so we add a day (end_time is exclusive so it
               -- won't include the first hour of the next day)
               DATEADD(DAY, 1, DATE_TRUNC('day', subscription_end_date)) AS end_time
          FROM workspace_operations.workspace_owner_subscriptions
          JOIN latest_subscription_record_timestamps
            USING (owner_pk)
            WHERE latest_record_timestamp = record_timestamp
      ),
      -- Link subscription records in order: the next subscription the user has (as long as it hasn't happened yet) will
      -- have a NULL start and end date, and its start date is really the end date of the previous subscription.
      SELECT owner_pk,
             subscription_id,
             LAG(end_time) OVER (PARTITION BY owner_pk ORDER BY start_time) AS prev_subscription_end_time,
             COALESCE(start_time, prev_subscription_end_time) AS start_time, -- Set start_time
             end_time,
             plan_code,
             record_timestamp,
             external_id,
             LEAD(subscription_id) OVER (PARTITION BY owner_pk ORDER BY start_time) AS next_subscription_id
        FROM latest_subscription_records;

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
