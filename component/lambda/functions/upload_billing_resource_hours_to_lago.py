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


def lambda_handler(lambda_event={}, _context=None):
    batch_hours = int(lambda_event.get("batch_hours", 6))
    assert batch_hours > 0

    last_hour_end = 0
    while last_hour_end is not None:
        first_hour_start = last_hour_end - batch_hours

        logger.info(
            f"Uploading {batch_hours} hours of billing events starting {-first_hour_start} hours ago"
        )

        #
        # Upload events by hour
        #
        all_events = cast(
            Iterator[tuple[ExternalSubscriptionId, SqlTimestamp, int]],
            redshift.query_raw(
                f"""
                SELECT external_subscription_id, hour_start, resource_count
                FROM workspace_operations.owner_resource_hours_with_subscriptions
                CROSS JOIN workspace_operations.workspace_update_events_summary
                WHERE external_subscription_id IS NOT NULL
                  AND DATEADD(HOUR, {first_hour_start}, last_complete_hour_start) <= hour_start
                  AND hour_start < DATEADD(HOUR, {last_hour_end}, last_complete_hour_start)
                ORDER BY hour_start DESC
                """
            ),
        )

        uploaded_events = 0
        for hour_start, hour_events in itertools.groupby(
            all_events, lambda event: event[1]
        ):
            logger.debug(f"Uploading events for hour {hour_start}")
            new_events, total_events = lago.upload_events(
                map(format_event, hour_events)
            )
            logger.info(f"Uploaded {new_events} / {total_events} for hour {hour_start}")
            if total_events == 0:
                logger.warning(f"No events found for hour {hour_start}")

            uploaded_events += new_events

        if uploaded_events == 0:
            logger.info(
                f"No events needed to be uploaded in the {batch_hours}-hour batch starting {-first_hour_start} hours ago! Finished."
            )
            last_hour_end = None
            break

        logger.warning(
            f"Uploaded {uploaded_events} events in {batch_hours}-hour batch starting {-first_hour_start} hours ago!"
        )
        last_hour_end = first_hour_start if uploaded_events > 0 else None


lambda_handler()
