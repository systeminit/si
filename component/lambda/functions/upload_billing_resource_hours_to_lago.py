from typing import Iterator, cast
from datetime import date, datetime, timedelta, timezone
import itertools
import boto3

from si_redshift import Redshift
from si_lago_api import LagoApi, ExternalSubscriptionId, LagoEvent
from si_logger import logger
from si_types import OwnerPk, SqlTimestamp


session = boto3.Session()
redshift = Redshift.from_env(session)
lago = LagoApi.from_env(session)


paul = OwnerPk("01GW0KXH4YJBWC7BTBAZ6ZR7EA")
billable_metric_code = "resource-hours"
cost_per_resource_hour = 0.007


def get_next_uninvoiced_month():
    # Decide what month to set price for
    today = date.today()
    next_uninvoiced_month = today.replace(day=1)
    # If we're still in the grace period at the beginning of the month, we want the price for the previous month
    if today.day < 5:
        next_uninvoiced_month = (next_uninvoiced_month - timedelta(days=1)).replace(
            day=1
        )
    return next_uninvoiced_month


def format_event(event: tuple[ExternalSubscriptionId, SqlTimestamp, int]) -> LagoEvent:
    [external_subscription_id, hour_start, resource_count] = event

    event_time = datetime.strptime(hour_start, "%Y-%m-%d %H:%M:%S").replace(
        tzinfo=timezone.utc
    )
    assert (
        event_time.minute == 0
        and event_time.second == 0
        and event_time.microsecond == 0
    )
    return {
        "transaction_id": f"{external_subscription_id}-{datetime.strftime(event_time, '%Y-%m-%d-%H')}",
        "external_subscription_id": external_subscription_id,
        "timestamp": event_time.timestamp(),
        "code": billable_metric_code,
        "properties": {
            "resource_hours": resource_count,
        },
    }


def lambda_handler(_event=None, _context=None):
    #
    # Upload events by hour
    #
    all_events = cast(
        Iterator[tuple[ExternalSubscriptionId, SqlTimestamp, int]],
        redshift.query_raw(
            """
            SELECT external_subscription_id, hour_start, resource_count
              FROM workspace_operations.owner_resource_hours_with_subscriptions
             WHERE hour_start <= (SELECT last_complete_event FROM workspace_operations.workspace_update_events_summary)
               AND external_subscription_id IS NOT NULL
             ORDER BY hour_start DESC
            """
        ),
    )

    for hour_start, hour_events in itertools.groupby(
        all_events, lambda event: event[1]
    ):
        logger.info(f"Uploading events for {hour_start}")
        for _event in hour_events:
            pass
        # lago.upload_events(map(format_event, hour_events))

    # lago.upload_events([format_event(event) for event in all_events])

    # #
    # # Upload prices
    # #
    # for [owner_pk, external_subscription_id, month, is_free] in cast(
    #     Iterator[tuple[OwnerPk, ExternalSubscriptionId, str, bool]],
    #     redshift.query_raw(
    #         """
    #         SELECT
    #             owner_billing_months.owner_pk,
    #             external_subscription_id,
    #             month,
    #             is_free
    #         FROM workspace_operations.owner_billing_months
    #         LEFT OUTER JOIN workspace_operations.latest_owner_subscriptions ON latest_owner_subscriptions.owner_pk = owner_billing_months.owner_pk
    #         WHERE month = :month
    #         AND owner_billing_months.owner_pk = :owner_pk
    #         """,
    #         month=str(get_next_uninvoiced_month()),
    #         owner_pk=paul,
    #     ),
    # ):
    #     # TODO only do this if it's active!
    #     json = {
    #         "subscription": {
    #             "plan_overrides": {
    #                 "charges": [
    #                     {
    #                         "resource-hours": {
    #                             "properties": {
    #                                 "amount": 0.4 if is_free else cost_per_resource_hour
    #                             }
    #                         }
    #                     }
    #                 ]
    #             }
    #         }
    #     }
    #     lago.put(f"/api/v1/subscriptions/{external_subscription_id}", json)


lambda_handler()
