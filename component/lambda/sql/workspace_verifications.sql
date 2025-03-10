--
-- Test queries to verify things went off correctly
--

CREATE SCHEMA IF NOT EXISTS workspace_verifications;

GRANT USAGE ON SCHEMA workspace_verifications TO lambda_user;
GRANT SELECT ON ALL TABLES IN SCHEMA workspace_verifications TO lambda_user;

-- All owner subscriptions must be in sequence:
-- * No gaps between subscriptions
-- * No overlaps between subscriptions
-- * Only one unbounded subscription at the end
-- * No invalid data (null IDs, start after end)
CREATE OR REPLACE VIEW workspace_verifications.subscription_issues AS
    SELECT
        *,
        CASE
            WHEN subscription_id IS NULL THEN 'null_subscription_id'
            WHEN external_id IS NULL THEN 'null_external_id'
            WHEN plan_code IS NULL THEN 'null_plan_code'
            WHEN start_time IS NULL THEN 'no_start_time' -- if the previous subscription has a start time, and 
            WHEN start_time > end_time THEN 'start_after_end'
            WHEN plan_code = 'launch_trial' AND end_time IS NULL THEN 'unbounded_free_trial'
            WHEN plan_code = 'launch_trial' AND prev_subscription_end_time IS NOT NULL THEN 'free_trial_not_first'
            WHEN plan_code <> 'launch_trial' AND prev_subscription_end_time IS NULL THEN 'free_trial_not_first'
            WHEN prev_subscription_end_time < start_time THEN 'gap_in_subscriptions'
            WHEN prev_subscription_end_time > start_time THEN 'overlapping_subscriptions'
            WHEN end_time < next_subscription_start_time THEN 'gap_in_subscriptions'
            WHEN end_time > next_subscription_start_time THEN 'overlapping_subscriptions'
            WHEN prev_subscription_end_time IS NULL AND start_time IS NULL AND next_subscription_id IS NOT NULL THEN 'multiple_unbounded_subscriptions'
            ELSE NULL
        END AS issue
    FROM workspace_operations.latest_owner_subscriptions
    WHERE issue IS NOT NULL;

CREATE OR REPLACE VIEW workspace_verifications.owner_subscriptions_summary AS
    SELECT
        owner_pk,
        COUNT(*) AS subscription_count,
        SUM(CASE WHEN plan_code = 'launch_trial' THEN 1 ELSE 0 END) AS free_trial_count,
        SUM(CASE WHEN end_time IS NULL THEN 1 ELSE 0 END) AS unbounded_subscription_count,
        MIN(start_time) AS first_subscription_start_time,
        MAX(start_time) AS last_subscription_start_time
      FROM workspace_operations.latest_owner_subscriptions
     GROUP BY owner_pk;

-- Owners must:
-- 1. Have subscriptions (we allow for one day of slop on this)
-- 2. Have a free trial subscription (we allow old free trials to be deleted if they can't possibly affect billing anymore)
-- 3. Have exactly one unbounded subscription (the last one)
CREATE OR REPLACE VIEW workspace_verifications.subscription_count_issues AS
    SELECT
        *,
        CASE
            WHEN subscription_count IS NULL AND first_timestamp < getdate() - INTERVAL '1 day' DAY THEN 'no_subscriptions'
            WHEN free_trial_count = 0 AND first_subscription_start_time >= getdate() - INTERVAL '2 months' THEN 'no_free_trial'
            WHEN unbounded_subscription_count = 0 THEN 'no_unbounded_subscription'
            WHEN unbounded_subscription_count > 1 THEN 'multiple_unbounded_subscriptions'
            ELSE NULL
        END AS issue
      FROM workspace_operations.owners
      LEFT OUTER JOIN workspace_verifications.owner_subscriptions_summary USING (owner_pk)
      WHERE issue IS NOT NULL;

-- We don't presently support workspaces changing ownership. Check this.
CREATE OR REPLACE VIEW workspace_verifications.workspace_issues AS
    WITH
        workspace_owners_linked AS (
            SELECT *, LAG(owner_pk) OVER (PARTITION BY workspace_id ORDER BY record_timestamp) AS prev_owner_pk
              FROM workspace_operations.workspace_owners
        )
    SELECT *, CASE WHEN prev_owner_pk <> owner_pk THEN 'owner_change' ELSE NULL END AS issue
      FROM workspace_owners_linked
     WHERE issue IS NOT NULL;

-- All events for an owner must be during a time with a subscription
CREATE OR REPLACE VIEW workspace_verifications.workspace_update_events_before_subscriptions AS
    WITH
        workspace_update_events_summary AS (
            SELECT workspace_id, MIN(event_timestamp) AS first_event, MAX(event_timestamp) AS last_event
              FROM workspace_update_events.workspace_update_events
             GROUP BY workspace_id
        )
    SELECT
        *,
        CASE
            WHEN owner_pk IS NULL THEN 'no_owner'
            WHEN first_event < first_subscription_start_time THEN 'event_before_subscription'
            ELSE NULL
        END AS issue
      FROM workspace_update_events_summary
      LEFT OUTER JOIN workspace_operations.workspace_owners USING (workspace_id)
      JOIN workspace_verifications.owner_subscriptions_summary USING (owner_pk)
      WHERE issue IS NOT NULL
WITH NO SCHEMA BINDING;

-- All resource hours must be associated with a subscription
CREATE OR REPLACE VIEW workspace_verifications.owner_resource_hours_without_subscriptions AS
    SELECT
        owner_pk,
        MIN(hour_start) AS first_event,
        MAX(hour_start) AS last_event,
        MAX(resource_count) AS max_resource_count,
        SUM(CASE WHEN external_subscription_id IS NULL THEN 1 ELSE 0 END) AS hours_without_subscription,
        MIN(CASE WHEN external_subscription_id IS NULL THEN hour_start ELSE NULL END) AS first_hour_without_subscription,
        MAX(CASE WHEN external_subscription_id IS NULL THEN hour_start ELSE NULL END) AS last_hour_without_subscription,
        MAX(CASE WHEN external_subscription_id IS NULL THEN resource_count ELSE NULL END) AS max_resource_count_without_subscription,
        CASE
            WHEN hours_without_subscription > 0 THEN 'events_without_subscription'
            ELSE NULL
        END AS issue
      FROM workspace_operations.owner_resource_hours_with_subscriptions
      GROUP BY owner_pk
     HAVING issue IS NOT NULL
WITH NO SCHEMA BINDING;
