create external schema spectrum_schema
from data catalog
database 'si-prod-data'
iam_role 'arn:aws:iam::$REDSHIFT_ACCOUNT:role/si-prod-redshift,arn:aws:iam::$S3_ACCOUNT:role/si-prod-glue-data-bucket';

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


-- Generates a list of all hours from Sep. 2024 until now
CREATE OR REPLACE VIEW workspace_operations.si_hours AS
    WITH RECURSIVE hour_generator (hour_start) AS (
        SELECT '2024-09-01 00:00:00'::timestamp
        UNION ALL (
            SELECT hour_start + INTERVAL '1 hour' HOUR
            FROM hour_generator
            WHERE hour_start < getdate()
        )
    )
    SELECT hour_start FROM hour_generator;

-- Get the resource counts for all workspaces anytime they change *and* at least once per hour
CREATE OR REPLACE VIEW workspace_operations.workspace_hourly_samples AS
    WITH
        -- Get all times at which any workspace for an owner has changed
        workspace_change_events AS (
            SELECT owner_pk, event_timestamp
            FROM workspace_operations.workspace_resource_counts
            JOIN workspace_operations.workspace_owners USING (workspace_id)
        ),
        -- List of all workspace owners
        owners AS (SELECT DISTINCT owner_pk FROM workspace_operations.workspace_owners),
        -- Get times at which we want to sample workspace resource counts
        --
        -- Using UNION instead of UNION ALL removes duplicates introduced when two
        -- workspaces change at the same time, as well as when a workspace changes at
        -- the exact start of an hour
        sample_times (owner_pk, hour_start, sample_time) AS (
            (SELECT owner_pk, DATE_TRUNC('hour', event_timestamp) AS hour_start, event_timestamp FROM workspace_change_events)
            UNION
            (SELECT owner_pk, hour_start, hour_start FROM workspace_operations.si_hours, owners)
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
        SELECT workspace_owner_subscriptions.*
          FROM workspace_operations.workspace_owner_subscriptions
          JOIN latest_subscription_record_timestamps
            USING (owner_pk)
            WHERE latest_record_timestamp = record_timestamp
      ),
      owners AS (SELECT DISTINCT owner_pk FROM workspace_operations.workspace_owners)
      SELECT owners.owner_pk,
             free_trial.external_id AS free_trial_external_subscription_id,
             free_trial.subscription_start_date AS free_trial_start_date,
             free_trial.subscription_end_date AS free_trial_end_date,
             subscription.external_id AS external_subscription_id,
             subscription.subscription_start_date,
             subscription.subscription_end_date,
             COALESCE(free_trial.record_timestamp, subscription.record_timestamp) AS subscription_record_timestamp
        FROM owners
        LEFT OUTER JOIN latest_subscription_records free_trial USING (owner_pk)
        LEFT OUTER JOIN latest_subscription_records subscription USING (owner_pk)
        WHERE free_trial.plan_code = 'launch_trial'
          AND subscription.plan_code = 'launch_pay_as_you_go';


-- Get owner_resource_hours with subscription data
CREATE OR REPLACE VIEW workspace_operations.owner_resource_hours_with_subscriptions AS
    WITH all_owner_resource_hours_with_subscriptions AS (
        SELECT owner_resource_hours.*,
            (CASE
                WHEN hour_start >= free_trial_end_date THEN external_subscription_id
                WHEN hour_start >= DATE_TRUNC('hour', free_trial_start_date) THEN free_trial_external_subscription_id
                ELSE NULL
            END) AS external_subscription_id
        FROM workspace_operations.owner_resource_hours
        LEFT OUTER JOIN workspace_operations.latest_owner_subscriptions USING (owner_pk)
    )
    -- Don't include data before SI launched unless it is attached to a subscription
    SELECT * FROM all_owner_resource_hours_with_subscriptions
     WHERE hour_start >= '2024-09-25'::timestamp OR external_subscription_id IS NOT NULL
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
-- Test queries to verify things went off correctly
--

CREATE SCHEMA workspace_verifications IF NOT EXISTS;

CREATE OR REPLACE VIEW workspace_verifications.owners_with_orphaned_resource_hours AS
    WITH
        owner_summary AS (
            SELECT
                owner_pk,
                MIN(hour_start) AS first_event,
                MAX(hour_start) AS last_event,
                MAX(resource_count) AS max_resource_count
            FROM owner_resource_hours_with_subscriptions
            GROUP BY owner_pk
        ),
        owner_resource_hours_without_subscriptions AS (
            SELECT
                owner_pk,
                MIN(hour_start) AS first_event_without_subscription,
                MAX(hour_start) AS last_event_without_subscription
            FROM owner_resource_hours_with_subscriptions
            WHERE external_subscription_id IS NULL GROUP BY owner_pk
        )
    SELECT
        owner_pk,
        first_event,
        first_event_without_subscription,
        last_event_without_subscription,
        last_event,
        max_resource_count,
        free_trial_external_subscription_id,
        external_subscription_id,
        free_trial_start_date,
        free_trial_end_date
    FROM owner_resource_hours_without_subscriptions
    JOIN owner_summary USING (owner_pk)
    LEFT OUTER JOIN latest_owner_subscriptions USING (owner_pk)
    ORDER BY first_event DESC
    WITH NO SCHEMA BINDING;

