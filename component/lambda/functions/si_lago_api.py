from collections.abc import Iterable
from itertools import islice
import os
import boto3
import json
from typing import Literal, NewType, TypeVar, TypedDict, Union, cast
import requests  # from pip._vendor import requests
import urllib.parse
from si_logger import logger

ExternalSubscriptionId = NewType("ExternalSubscriptionId", str)


class LagoEvent(TypedDict):
    transaction_id: str
    external_subscription_id: ExternalSubscriptionId
    timestamp: float
    code: str
    properties: dict[str, int]


T = TypeVar("T")


class LagoErrorResponse(TypedDict):
    code: str
    error_details: dict[str, dict[str, list[str]]]


def batch(iterable: Iterable[T], n):
    "Batch data into lists of length n. The last batch may be shorter."
    # batched('ABCDEFG', 3) --> ABC DEF G
    it = iter(iterable)
    while True:
        batch = list(islice(it, n))
        if not batch:
            return
        yield batch


class LagoHTTPError(Exception):
    def __init__(self, *args, json, **kwargs):
        super().__init__(*args, **kwargs)
        self.json = json


class LagoApi:
    @classmethod
    def from_env(cls, session: boto3.Session):
        lago_api_url = os.environ["LAGO_API_URL"]
        lago_api_token = os.environ.get("LAGO_API_TOKEN")
        if lago_api_token is None:
            lago_api_token_arn = os.environ["LAGO_API_TOKEN_ARN"]

            secretsmanager = session.client(service_name="secretsmanager")
            secret = secretsmanager.get_secret_value(SecretId=lago_api_token_arn)
            lago_api_token = json.loads(secret["SecretString"])["LAGO_API_TOKEN"]

        return LagoApi(lago_api_url, lago_api_token)

    def __init__(self, lago_api_url: str, lago_api_token: str):
        self.lago_api_url = lago_api_url
        self.lago_api_token = lago_api_token

    def request(self, method: str, path: str, **kwargs):
        response = requests.request(
            method,
            urllib.parse.urljoin(self.lago_api_url, path),
            headers={"Authorization": f"Bearer {self.lago_api_token}"},
            **kwargs,
        )
        try:
            response.raise_for_status()
        except requests.HTTPError as e:
            raise LagoHTTPError(
                f"{e.response.status_code} for {method} {path}: {e.response.text}",
                json=e.response.json(),
            ) from e

        return response

    def get(self, path: str):
        return self.request("GET", path)

    def delete(self, path: str):
        return self.request("DELETE", path)

    def put(self, path: str, json):
        return self.request("PUT", path, json=json)

    def post(self, path: str, json):
        return self.request("POST", path, json=json)

    def upload_events(self, events: Iterable[LagoEvent]):
        """
        Uploads events to Lago, batching them in groups of 100 (Lago's maximum).

        If a batch fails due to an event already having been uploaded, we retry the ones that
        try to upload each event
        individually.

        Exceptions:
        - HTTPError: Any HTTP error response except 422 (which indicates the event already exists)
        will be thrown.
        """

        for event_batch in batch(events, 100):
            try:
                self.post("/api/v1/events/batch", {"events": event_batch})

            except LagoHTTPError as e:

                # If the batch failed because some events were already uploaded, retry the rest
                if [e.json.get("status"), e.json.get("code")] != [422, "validation_errors"]:
                    raise

                for error in e.json["error_details"].values():
                    if error.get("transaction_id") != ["value_already_exist"]:
                        raise

                # Retry the events that did not fail
                logger.warning(
                    f"Events already existed: {[event_batch[int(i)]['transaction_id'] for i in e.json["error_details"].keys()]}"
                )
                retry_event_batch = [
                    event
                    for (i, event) in enumerate(event_batch)
                    if str(i) not in e.json["error_details"].keys()
                ]

                if len(retry_event_batch) > 0:
                    logger.warning(
                        f"Reuploading remaining {len(retry_event_batch)} events."
                    )

                    # Retry the events that didn't fail. Any failure here is a real error
                    self.post("/api/v1/events/batch", {"events": retry_event_batch})
