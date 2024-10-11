from datetime import datetime, timedelta
from typing import Iterator, Literal, NotRequired, Optional, TypedDict, cast
import boto3

from si_logger import logger
from si_redshift import Redshift
from si_lago_api import (
    LagoApi,
    ExternalSubscriptionId,
    LagoHTTPError,
    LagoInvoicesResponse,
    LagoSubscription,
)
from si_types import OwnerPk


session = boto3.Session()
redshift = Redshift.from_env(session)
lago = LagoApi.from_env(session)

paul = OwnerPk("01GW0KXH4YJBWC7BTBAZ6ZR7EA")
plan_code = "launch_pay_as_you_go"
billable_metric_code = "resource-hours"
cost_per_resource_hour = 0.007


def get_invoice_month():
    """
    Get the start of the month for the next invoice we will send out.

    Prices will be set based on the user's usage during this month.
    """
    now = datetime.now()
    day_in_invoice_month = now
    if now.day > 5:
        day_in_invoice_month = day_in_invoice_month - timedelta(days=6)
    return day_in_invoice_month.replace(
        day=1, hour=0, minute=0, second=0, microsecond=0
    )


def get_subscription(
    external_subscription_id: ExternalSubscriptionId,
) -> LagoSubscription:
    try:
        return cast(
            LagoSubscription,
            lago.get(f"/api/v1/subscriptions/{external_subscription_id}").json()[
                "subscription"
            ],
        )
    except LagoHTTPError as e:
        if e.json.get("status") == 404:
            logger.debug(
                f"Subscription {external_subscription_id} is not active. Getting pending version ..."
            )
            return cast(
                LagoSubscription,
                lago.get(
                    f"/api/v1/subscriptions/{external_subscription_id}?status=pending"
                ).json()["subscription"],
            )
        raise


def lambda_handler(_event=None, _context=None):
    #
    # Upload prices
    #
    for [owner_pk, external_subscription_id, month, is_free] in cast(
        Iterator[tuple[OwnerPk, ExternalSubscriptionId, str, bool]],
        redshift.query_raw(
            """
            SELECT
                owner_pk,
                external_subscription_id,
                month,
                is_free
            FROM workspace_operations.owner_billing_months
            LEFT OUTER JOIN workspace_operations.latest_owner_subscriptions USING(owner_pk)
            WHERE month = :month
            ORDER BY owner_pk
            """,
            month=str(get_invoice_month()),
        ),
    ):
        if external_subscription_id is None:
            logger.warning(f"Owner {owner_pk} has no subscriptions. Skipping ...")
            continue

        # Decide the price for this user.
        amount = 0 if is_free else cost_per_resource_hour

        # Get the current version of the subscription.
        subscription = get_subscription(external_subscription_id)
        if subscription["plan_code"] != plan_code:
            logger.warning(
                f"Subscription {external_subscription_id} is not on the {plan_code} plan. Assuming it has been set up custom, so not updating price ..."
            )
            continue
        assert subscription["plan_code"] == plan_code

        # If the user's price is already correct, don't set it again.
        charge = next(
            charge
            for charge in subscription["plan"]["charges"]
            if charge["billable_metric_code"] == "resource-hours"
        )
        current_amount = charge["properties"].get("amount")
        assert current_amount is not None
        if float(current_amount) == amount:
            logger.debug(
                f"Subscription {external_subscription_id}'s price is already set to {amount}. Not updating ..."
            )
            continue

        # TODO what if the plan has been terminated or replaced, but the last invoice is pending?
        # We still want to modify the subscription. ... can we even do that?

        # Set the subscription's price.
        logger.info(
            f"Setting subscription {external_subscription_id}'s price from {current_amount} to {amount} ..."
        )
        lago.put(
            f"/api/v1/subscriptions/{external_subscription_id}",
            {
                "status": subscription["status"],
                "subscription": {
                    "plan_overrides": {
                        "charges": [
                            {
                                "billable_metric_id": charge["lago_billable_metric_id"],
                                "charge_model": "standard",
                                "properties": {"amount": str(amount)},
                            }
                        ]
                    }
                },
            },
        )


lambda_handler()
