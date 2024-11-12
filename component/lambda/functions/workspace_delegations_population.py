from collections.abc import Iterable
import json
from typing import NotRequired, Optional, TypedDict, cast, overload
import time
from datetime import datetime
from si_lambda import SiLambda, SiLambdaEnv
from si_types import WorkspaceId, OwnerPk, SqlTimestamp
from si_lago_api import ExternalSubscriptionId, LagoSubscription, LagoSubscriptionsResponse, IsoTimestamp
import logging
from itertools import groupby

class WorkspaceDelegationsPopulationEnv(SiLambdaEnv):
    SI_OWNER_PKS: NotRequired[Optional[list[OwnerPk]]]

class WorkspaceDelegationsPopulation(SiLambda):
    def __init__(self, event: WorkspaceDelegationsPopulationEnv):
        super().__init__(event)
        self.owner_pks = event.get("SI_OWNER_PKS")
        if self.owner_pks is not None:
            assert len(self.owner_pks) > 0, "SI_OWNER_PKS must be non-empty. Did you mean to not set it?"

    def update_subscriptions(self, current_timestamp: SqlTimestamp):
        # Query all owner subscriptions and compare to what's in Lago
        latest_owner_subscriptions = cast(Iterable['LatestOwnerSubscription'], self.redshift.query("""
            SELECT owner_pk, subscription_id, subscription_start_date, subscription_end_date, plan_code, external_id
              FROM workspace_operations.owners
              LEFT OUTER JOIN workspace_operations.latest_owner_subscriptions USING (owner_pk)
             ORDER BY owner_pk, start_time
        """))

        # Start all the inserts at once (if any)
        subscription_updates = [
            self.update_owner_subscriptions(owner_pk, current_timestamp, list(si_subscriptions), self.get_owner_lago_subscriptions(owner_pk))
            for owner_pk, si_subscriptions in groupby(latest_owner_subscriptions, lambda sub: sub['owner_pk'])
            if self.owner_pks is None or owner_pk in self.owner_pks
        ]
        # Return the completed inserts
        return [
            result.wait_for_complete()['Status']
            for result in subscription_updates
            if result is not None
        ]

    def update_owner_subscriptions(self, owner_pk: OwnerPk, current_timestamp: SqlTimestamp, si_subscriptions: list['LatestOwnerSubscription'], lago_subscriptions: Iterable[LagoSubscription]):
        lago_subscriptions = list(lago_subscriptions)
        # Get Lago subscriptions for the owner
        lago_subscriptions_by_id = {
            sub['external_id']: self.lago_to_si_subscription(owner_pk, sub)
            for sub in lago_subscriptions
            # Skip subs that were started and terminated the same day
            if not (sub['status'] == 'terminated' and iso_to_days(sub.get('started_at')) == iso_to_days(sub.get('terminated_at')))
        }

        # Compare and decide whether to update subscriptions in SI
        should_update = False
        if len(si_subscriptions) == 0:
            if len(lago_subscriptions_by_id) != 0:
                logging.info(f"New owner {owner_pk}! Adding {lago_subscriptions_by_id.keys()}")
                should_update = True
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
                    logging.info(f"Owner {owner_pk} has a new subscription {external_id} in Lago! Adding to SI.")
                    should_update = True

        if should_update:
            return self.start_inserting_owner_subscriptions(owner_pk, lago_subscriptions_by_id.values(), current_timestamp)

    def start_inserting_owner_subscriptions(self, owner_pk: OwnerPk, subscriptions: Iterable['LatestOwnerSubscription'], timestamp):
        def value_row(sub: 'LatestOwnerSubscription'):
            subscription_start_date = 'NULL' if sub['subscription_start_date'] is None else f"'{sub['subscription_start_date']}'" 
            subscription_end_date = "NULL" if sub['subscription_end_date'] is None else f"'{sub['subscription_end_date']}'"
            return f"('{owner_pk}', '{sub['subscription_id']}', {subscription_start_date}, {subscription_end_date}, '{sub['plan_code']}', '{timestamp}', '{sub['external_id']}')"

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
            'subscription_start_date': convert_iso_to_datetime(lago_sub.get('started_at')),
            'subscription_end_date': convert_iso_to_datetime(lago_sub.get('ending_at')),
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

    def insert_missing_workspaces(self, current_timestamp: SqlTimestamp):
        missing_workspace_inserts = [
            [
                workspace_id,
                self.start_inserting_workspace(
                    workspace_id,
                    self.auth_api.owner_workspaces(workspace_id)["workspaceOwnerId"],
                    current_timestamp
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
        return {
            workspace_id: insert.wait_for_complete()['Status']
            for workspace_id, insert in missing_workspace_inserts
        }

    def start_inserting_workspace(self, workspace_id: WorkspaceId, workspace_owner_id: OwnerPk, timestamp):
        # Prepare the columns and values for the workspace_owners table
        columns = ["owner_pk", "workspace_id", "record_timestamp"]
        values = [
            f"'{workspace_owner_id}'", 
            f"'{workspace_id}'", 
            f"'{timestamp}'"  # Use the provided timestamp
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

    def run(self):
        # Get the current timestamp for record insertion
        current_timestamp = cast(SqlTimestamp, time.strftime('%Y-%m-%d %H:%M:%S'))

        inserted_workspaces = self.insert_missing_workspaces(current_timestamp)
        updated_subscriptions = self.update_subscriptions(current_timestamp)

        return {
            'statusCode': 200,
            'body': json.dumps({
                'inserted_workspaces': inserted_workspaces,
                'updated_subscriptions': updated_subscriptions,
            })
        }

# Result of the latest_owner_subscriptions query
class LatestOwnerSubscription(TypedDict):
    owner_pk: OwnerPk
    subscription_id: str
    subscription_start_date: Optional[SqlTimestamp]
    subscription_end_date: Optional[SqlTimestamp]
    plan_code: str
    external_id: ExternalSubscriptionId


# Convert ISO 8601 timestamp to the required format
@overload
def convert_iso_to_datetime(iso_str: IsoTimestamp) -> SqlTimestamp: ...
@overload
def convert_iso_to_datetime(iso_str: None) -> None: ...
@overload
def convert_iso_to_datetime(iso_str: Optional[IsoTimestamp]) -> Optional[SqlTimestamp]: ...
def convert_iso_to_datetime(iso_str: Optional[IsoTimestamp]):
    if iso_str is None:
        return None
    return datetime.strptime(iso_str, '%Y-%m-%dT%H:%M:%SZ').strftime('%Y-%m-%d %H:%M:%S')

@overload
def iso_to_days(iso_str: IsoTimestamp) -> str: ...
@overload
def iso_to_days(iso_str: None) -> None: ...
@overload
def iso_to_days(iso_str: Optional[IsoTimestamp]) -> Optional[str]: ...
def iso_to_days(iso_str: Optional[IsoTimestamp]):
    if iso_str is None:
        return None
    return datetime.strptime(iso_str, '%Y-%m-%dT%H:%M:%SZ').strftime('%Y-%m-%d')

def lambda_handler(event = {}, _context = None):
    WorkspaceDelegationsPopulation(event).run()

