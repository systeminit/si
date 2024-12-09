from datetime import datetime, timedelta
import logging
from typing import Iterable, cast

from si_lambda import SiLambda
from si_lago_api import (
    ExternalSubscriptionId,
    LagoHTTPError,
    LagoSubscription,
)
from si_types import OwnerPk

pay_as_you_go_plan_code = "launch_pay_as_you_go"
billable_metric_code = "resource-hours"
cost_per_resource_hour = 0.007


class BillingSetPrices(SiLambda):
    def get_invoice_month(self):
        """
        Get the start of the month for the next invoice we will send out.

        Prices will be set based on the user's usage during the month that will be billed next.
        In most cases that is the current month, but for the first few days of the months we
        need to keep the price based on the previous month until the invoice goes out because
        the invoice will automatically update to include the price. When the invoice is
        finalized (after a grace period of a few days), we once again set the price based on
        the current month's usage.
        """
        now = datetime.now()
        day_in_invoice_month = now
        if now.day > 5:
            day_in_invoice_month = day_in_invoice_month - timedelta(days=6)
        return day_in_invoice_month.replace(
            day=1, hour=0, minute=0, second=0, microsecond=0
        )

    def get_subscription(self, external_subscription_id: ExternalSubscriptionId):
        try:
            return cast(
                LagoSubscription,
                self.lago.get(f"/api/v1/subscriptions/{external_subscription_id}").json()[
                    "subscription"
                ],
            )

        except LagoHTTPError as e:
            if e.json and e.json.get("status") != 404:
                raise
            try:
                logging.debug(
                    f"Subscription {external_subscription_id} is not active. Getting pending version ..."
                )
                return cast(
                    LagoSubscription,
                    self.lago.get(
                        f"/api/v1/subscriptions/{external_subscription_id}?status=pending"
                    ).json()["subscription"],
                )

            except LagoHTTPError as e:
                if e.json and e.json.get("status") != 404:
                    raise

                try:
                    logging.debug(
                        f"Subscription {external_subscription_id} is not active or pending. Getting terminated version ..."
                    )
                    return cast(
                        LagoSubscription,
                        self.lago.get(
                            f"/api/v1/subscriptions/{external_subscription_id}?status=terminated"
                        ).json()["subscription"],
                    )

                except LagoHTTPError as e:
                    if e.json and e.json.get("status") != 404:
                        raise

                    logging.warning(
                        f"Subscription {external_subscription_id} no longer exists or is not active, pending, or terminated. Skipping ..."
                    )
                    return None

    def run(self):

        #
        # Upload prices
        #
        for [owner_pk, _month, pay_as_you_go_subscription_id, is_free] in cast(
            Iterable[tuple[OwnerPk, str, ExternalSubscriptionId, bool]],
            self.redshift.query_raw(
                """
                SELECT owner_pk, month, external_id, is_free
                FROM workspace_operations.owner_billing_months
                LEFT OUTER JOIN workspace_operations.latest_owner_subscriptions USING (owner_pk)
                WHERE month = :month AND plan_code = 'launch_pay_as_you_go'
                ORDER BY owner_pk
                """,
                month=str(self.get_invoice_month()),
            ),
        ):
            if pay_as_you_go_subscription_id is None:
                logging.warning(f"Owner {owner_pk} has no pay-as-you-go subscriptions. Skipping ...")
                continue

            # Decide the price for this user.
            amount = 0 if is_free else cost_per_resource_hour

            # Get the current version of the subscription.
            subscription = self.get_subscription(pay_as_you_go_subscription_id)
            if subscription is None:
                if amount != 0:
                    logging.error("Non-free subscription {pay_as_you_go_subscription_id} is no longer in Lago!")
                continue

            assert subscription["plan_code"] == pay_as_you_go_plan_code

            # If the user's price is already correct, don't set it again.
            charge = next(
                charge
                for charge in subscription["plan"]["charges"]
                if charge["billable_metric_code"] == "resource-hours"
            )
            current_amount = charge["properties"].get("amount")
            assert current_amount is not None
            if float(current_amount) == amount:
                logging.debug(
                    f"Subscription {pay_as_you_go_subscription_id}'s price is already set to {amount}. Not updating ..."
                )
                continue

            # TODO what if the plan has been terminated or replaced, but the last invoice is pending?
            # We still want to modify the subscription. ... can we even do that?

            # Set the subscription's price.
            path = f"/api/v1/subscriptions/{pay_as_you_go_subscription_id}"
            payload = {
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
                }
            logging.info(
                f"Setting subscription {pay_as_you_go_subscription_id}'s price from {current_amount} to {amount} ..."
            )
            if self.dry_run:
                logging.info("Would PUT {path} with payload: {payload}")
            else:
                self.lago.put(path, payload)


lambda_handler = BillingSetPrices.lambda_handler