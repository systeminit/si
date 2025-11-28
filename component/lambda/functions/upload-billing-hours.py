from typing import Iterable, NotRequired, cast
from datetime import date, datetime, timedelta, timezone
import itertools
import logging

from si_lambda import SiLambda, SiLambdaEnv
from si_lago_api import ExternalSubscriptionId, LagoEvent
from si_types import SqlTimestamp

billable_metric_code = "resource-hours"
cost_per_resource_hour = 0.007

class UploadBillingHoursEnv(SiLambdaEnv):
    batch_hours: NotRequired[int]
    end_hours_ago: NotRequired[int]

class UploadBillingHours(SiLambda):
    def __init__(self, event: UploadBillingHoursEnv):
        super().__init__(event)
        self.batch_hours = int(event.get("batch_hours", 6))
        self.end_hours_ago = int(event.get("end_hours_ago", 0))
        assert self.batch_hours > 0

    def get_next_uninvoiced_month(self):
        # Decide what month to set price for
        today = date.today()
        next_uninvoiced_month = today.replace(day=1)
        # If we're still in the grace period at the beginning of the month, we want the price for the previous month
        if today.day < 5:
            next_uninvoiced_month = (next_uninvoiced_month - timedelta(days=1)).replace(day=1)
        return next_uninvoiced_month

    def format_event(self, event: tuple[ExternalSubscriptionId, SqlTimestamp, int]) -> LagoEvent:
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

    def run(self):
        last_hour_end = -self.end_hours_ago
        while last_hour_end is not None:
            first_hour_start = last_hour_end - self.batch_hours

            logging.info(
                f"Uploading {self.batch_hours} hours of billing events starting {-first_hour_start} hours ago"
            )

            #
            # Upload events by hour
            #
            all_events = cast(
                Iterable[tuple[ExternalSubscriptionId, SqlTimestamp, int]],
                self.redshift.query_raw(
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
                logging.debug(f"Uploading events for hour {hour_start}")
                new_events, total_events = self.lago.upload_events(
                    map(self.format_event, hour_events), dry_run=self.dry_run
                )
                logging.info(f"Uploaded {new_events} / {total_events} for hour {hour_start}")
                if total_events == 0:
                    logging.warning(f"No events found for hour {hour_start}")

                uploaded_events += new_events

            if uploaded_events == 0:
                logging.info(
                    f"No events needed to be uploaded in the {self.batch_hours}-hour batch starting {-first_hour_start} hours ago! Finished."
                )
                last_hour_end = None
                break

            logging.info(
                f"Uploaded {uploaded_events} events in {self.batch_hours}-hour batch starting {-first_hour_start} hours ago!"
            )
            last_hour_end = first_hour_start if uploaded_events > 0 else None

lambda_handler = UploadBillingHours.lambda_handler