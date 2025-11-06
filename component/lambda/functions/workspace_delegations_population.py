from collections.abc import Iterable
import json
from typing import NotRequired, Optional, TypedDict, Union, cast, overload
import time
from si_lambda import SiLambda, SiLambdaEnv
from si_types import WorkspaceId, OwnerPk, SqlDatetime, iso_to_sql_days, iso_to_sql_datetime
from si_lago_api import ExternalSubscriptionId, LagoSubscription, LagoSubscriptionsResponse, IsoTimestamp
import logging
from itertools import groupby
from pprint import pprint

class WorkspaceDelegationsPopulationEnv(SiLambdaEnv):
    owner_pks: NotRequired[Optional[list[OwnerPk]]]

class WorkspaceDelegationsPopulation(SiLambda):
    def __init__(self, event: WorkspaceDelegationsPopulationEnv):
        super().__init__(event)
        self.owner_pks = event.get("owner_pks")
        if self.owner_pks is not None:
            assert len(self.owner_pks) > 0, "owner_pks must be non-empty. Did you mean to not set it?"

    def run(self):
        inserted_workspaces = self.insert_missing_workspaces()
        updated_subscriptions = self.update_subscriptions()

        return {
            'statusCode': 200,
            'body': json.dumps({
                'inserted_workspaces': inserted_workspaces,
                'updated_subscriptions': updated_subscriptions,
            })
        }

    def update_subscriptions(self):
        # Query all owner subscriptions and compare to what's in Lago
        subscription_updates = []
        for owner_pk, si_subscriptions in self.current_owner_subscriptions():
            lago_subscriptions = self.get_owner_lago_subscriptions(owner_pk)
            subscription_updates.append(
                self.update_owner_subscriptions(owner_pk, si_subscriptions, lago_subscriptions))

        # Return the completed inserts
        results = [
            update.wait_for_complete()
            for update in subscription_updates
            if update is not None
        ]
        logging.info(f"Subscription update complete: {sum([result['ResultRows'] for result in results])} subscriptions updated.")
        return [result['Status'] for result in results]

    # Get the current list of owners and their subscriptions in Redshift.
    def current_owner_subscriptions(self):
        latest_owner_subscriptions = self.redshift.query(f"""
            SELECT owner_pk, subscription_id, subscription_start_date, subscription_end_date, plan_code, external_id
              FROM workspace_operations.owners
              LEFT OUTER JOIN workspace_operations.latest_owner_subscriptions USING (owner_pk)
             ORDER BY owner_pk, start_time
        """)

        total_subscriptions = latest_owner_subscriptions.wait_for_complete()['ResultRows']

        # Start all the inserts at once (if any)
        started_at = time.time()
        last_report = started_at
        processed_subscriptions = 0
        for owner_pk, si_subscriptions_iter in groupby(
            cast(Iterable[Union[LatestOwnerSubscription, OwnerWithoutSubscriptions]], latest_owner_subscriptions),
            lambda sub: sub['owner_pk']
        ):
            si_subscriptions = list(si_subscriptions_iter)
            # Skip owners not in the filter list, if a filter list is specified
            if self.owner_pks is not None and owner_pk not in self.owner_pks:
                total_subscriptions -= len(si_subscriptions)

            # If the owner existed, but has no subscriptions, yield the owner with an empty list
            elif len(si_subscriptions) >= 1 and si_subscriptions[0]['subscription_id'] is None:
                total_subscriptions -= len(si_subscriptions)
                yield owner_pk, []

            # Yield the owner and their subscriptions
            else:
                processed_subscriptions += len(si_subscriptions)
                yield owner_pk, cast(list[LatestOwnerSubscription], si_subscriptions)

            if time.time() - last_report > 5:
                logging.info(f"{processed_subscriptions} / {total_subscriptions} subscriptions retrieved from Lago after {time.time() - started_at}s.")
                last_report = time.time()

    def update_owner_subscriptions(self, owner_pk: OwnerPk, si_subscriptions: list['LatestOwnerSubscription'], lago_subscriptions: list[LagoSubscription]):
        # If a subscription appears multiple times, the latest one is the valid one
        lago_subscriptions.sort(key=lambda sub: sub['created_at'])

        # Get Lago subscriptions for the owner
        lago_subscriptions_by_id = {
            sub['external_id']: self.lago_to_si_subscription(owner_pk, sub)
            for sub in lago_subscriptions
            # Skip subs that were started and terminated the same day
            if not (sub['status'] == 'terminated' and iso_to_sql_days(sub.get('started_at')) == iso_to_sql_days(sub.get('terminated_at')))
        }

        # Compare and decide whether to update subscriptions in SI
        should_update = False
        if len(si_subscriptions) == 0:
            if len(lago_subscriptions_by_id) != 0:
                should_update = True
                logging.info(f"New owner {owner_pk}! Adding {lago_subscriptions_by_id.keys()}")
        elif len(lago_subscriptions_by_id) == 0:
            logging.error(f"Owner {owner_pk} has had all subscriptions removed from Lago!")
        else:
            # Look for modified or removed subscriptions
            for si_sub in si_subscriptions:
                lago_sub = lago_subscriptions_by_id.get(si_sub['external_id'])
                if si_sub != lago_sub:
                    should_update = True
                    if lago_sub is None:
                        logging.error(f"Owner {owner_pk}'s subscription {si_sub['external_id']} has been removed from Lago! Removing from SI.")
                    else:
                        logging.info(f"Owner {owner_pk}'s subscription {si_sub['external_id']} has changed in Lago! Updating in SI.")

            # Look for new subscriptions
            si_subscription_ids = set([sub['external_id'] for sub in si_subscriptions])
            for external_id in lago_subscriptions_by_id.keys():
                if external_id not in si_subscription_ids:
                    should_update = True
                    logging.info(f"Owner {owner_pk} has a new subscription {external_id} in Lago! Adding to SI.")

        if should_update:
            return self.start_inserting_owner_subscriptions(owner_pk, lago_subscriptions_by_id.values())

    def start_inserting_owner_subscriptions(self, owner_pk: OwnerPk, subscriptions: Iterable['LatestOwnerSubscription']):
        def value_row(sub: 'LatestOwnerSubscription'):
            subscription_start_date = 'NULL' if sub['subscription_start_date'] is None else f"'{sub['subscription_start_date']}'" 
            subscription_end_date = "NULL" if sub['subscription_end_date'] is None else f"'{sub['subscription_end_date']}'"
            return f"('{owner_pk}', '{sub['subscription_id']}', {subscription_start_date}, {subscription_end_date}, '{sub['plan_code']}', '{self.start_timestamp_sql}', '{sub['external_id']}')"

        value_rows = ",\n  ".join([value_row(sub) for sub in subscriptions])
        sql = f'INSERT INTO workspace_operations.workspace_owner_subscriptions\n  (owner_pk, subscription_id, subscription_start_date, subscription_end_date, plan_code, record_timestamp, external_id) VALUES\n  {value_rows}'

        if self.dry_run:
            logging.info(f"DRY RUN: Inserting into workspace_owner_subscriptions: {sql}")
        else:
            return self.redshift.start_executing(sql)

    def lago_to_si_subscription(self, owner_pk: OwnerPk, lago_sub: LagoSubscription) -> 'LatestOwnerSubscription':
        return {
            'owner_pk': owner_pk,
            'subscription_id': lago_sub['lago_id'],
            'subscription_start_date': iso_to_sql_datetime(lago_sub.get('started_at')),
            'subscription_end_date': iso_to_sql_datetime(lago_sub.get('ending_at')),
            'plan_code': lago_sub['plan_code'],
            'external_id': lago_sub['external_id']
        }

    def get_all_lago_subscriptions(self):
        url = "/api/v1/subscriptions?status[]=pending&status[]=active&status[]=terminated&status[]=active"
        page = cast(LagoSubscriptionsResponse, self.lago.get(url).json())
        logging.debug(f"Lago subscriptions: page {page['meta']['current_page']}/{page['meta']['total_pages']} with {len(page['subscriptions'])}/{page['meta']['total_count']} subscriptions")
        yield from page['subscriptions']

        while page['meta'].get('next_page') is not None:
            page = cast(LagoSubscriptionsResponse, self.lago.get(f"{url}&page={page['meta'].get('next_page')}").json())
            logging.debug(f"Lago subscriptions: page {page['meta']['current_page']}/{page['meta']['total_pages']} with {len(page['subscriptions'])}/{page['meta']['total_count']} subscriptions")
            yield from page['subscriptions']

    def get_owner_lago_subscriptions(self, owner_pk: OwnerPk):
        subs = cast(LagoSubscriptionsResponse, self.lago.get(
            f"/api/v1/subscriptions?external_customer_id={owner_pk}&status[]=pending&status[]=active&status[]=terminated&status[]=active"
        ).json())
        assert(subs['meta']['total_count'] == len(subs['subscriptions']))
        return subs['subscriptions']

    def insert_missing_workspaces(self):
        missing_workspace_inserts = [
            [
                workspace_id,
                self.start_inserting_workspace(
                    workspace_id,
                    self.auth_api.owner_workspaces(workspace_id)["workspaceOwnerId"],
                )
            ]
            for [workspace_id] in cast(Iterable[tuple[WorkspaceId]], self.redshift.query_raw("""
                SELECT DISTINCT workspace_id
                    FROM workspace_update_events.workspace_update_events
                    LEFT OUTER JOIN workspace_operations.workspace_owners USING (workspace_id)
                    WHERE workspace_owners.workspace_id IS NULL
                    LIMIT 50
            """))
        ]
        results = {
            workspace_id: insert.wait_for_complete()
            for workspace_id, insert in missing_workspace_inserts
        }
        logging.info(f"Insert missing workspaces complete: {sum([result['ResultRows'] for result in results.values()])} workspaces inserted.")
        return {
            workspace_id: result['Status']
            for workspace_id, result in results.items()
        }

    def start_inserting_workspace(self, workspace_id: WorkspaceId, workspace_owner_id: OwnerPk):
        # Prepare the columns and values for the workspace_owners table
        columns = ["owner_pk", "workspace_id", "record_timestamp"]
        values = [
            f"'{workspace_owner_id}'", 
            f"'{workspace_id}'", 
            f"'{self.start_timestamp_sql}'"  # Use the provided timestamp
        ]

        # Construct the SQL query for inserting into workspace_owners
        columns_str = ", ".join(columns)
        values_str = ", ".join(values)

        sql = f"""
            INSERT INTO "workspace_operations"."workspace_owners" 
            ({columns_str}) 
            VALUES ({values_str});
        """
        if self.dry_run:
            logging.info(f"DRY RUN: Would insert into workspace_owner_subscriptions: {sql}")
        else:
            logging.info(f"Inserting into workspace_owner_subscriptions: {sql}")
            return self.redshift.start_executing(sql)

# Result of the latest_owner_subscriptions query

class LatestOwnerSubscription(TypedDict):
    owner_pk: OwnerPk
    subscription_id: str
    subscription_start_date: Optional[SqlDatetime]
    subscription_end_date: Optional[SqlDatetime]
    plan_code: str
    external_id: ExternalSubscriptionId

class OwnerWithoutSubscriptions(TypedDict):
    owner_pk: OwnerPk
    subscription_id: None
    plan_code: None
    external_id: None

lambda_handler = WorkspaceDelegationsPopulation.lambda_handler

