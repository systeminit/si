from collections.abc import Iterable
from itertools import islice
import logging
import sys
import urllib.parse
from pip._vendor import requests
from pip._vendor.requests.exceptions import HTTPError
from pip._vendor.requests.models import Response
from typing import Any, Literal, NotRequired, Optional, Type, TypeVar, TypedDict, Union, cast, overload

from si_types import IsoTimestamp

class PosthogApi:
    def __init__(self, posthog_api_url: str, posthog_project_api_key: str):
        self.posthog_api_url = posthog_api_url
        self.posthog_project_api_key = posthog_project_api_key

    def request(self, method: str, path: str, **kwargs):
        logging.debug(f"Posthog API request: {method} {path}")
        response = requests.api.request(
            method,
            urllib.parse.urljoin(self.posthog_api_url, path),
            **kwargs,
        )
        
        try:
            response.raise_for_status()
        except HTTPError as e:
            message = f"{e.response.status_code} for {method} {path}: {e.response.text}"
            try:
                json = cast(PosthogApi.HTTPError.ResponseJson, e.response.json())
            except:
                logging.warning("Failed to parse JSON from Posthog API response.", exc_info=sys.exc_info())
                json = None
            raise PosthogApi.HTTPError(message, response=e.response, json=json) from e
        return response

    @overload
    def post(self, path: Literal["/i/v0/e"], json: "EventRequest"): ...
    @overload
    def post(self, path: Literal["/batch"], json: "BatchRequest"): ...
    def post(self, path: str, json):
        return self.request("POST", path, json = {
            "api_key": self.posthog_project_api_key,
            **json
        })

    class EventRequest(TypedDict):
        event: str
        distinct_id: str
        properties: dict[str, Any]
        timestamp: NotRequired[IsoTimestamp]

    class BatchRequest(TypedDict):
        batch: list["PosthogApi.BatchEvent"]
        historical_migration: NotRequired[bool]
    class BatchEvent(TypedDict):
        event: str
        properties: dict[str, Any]
        timestamp: NotRequired[IsoTimestamp]

    class HTTPError(HTTPError):
        class GenericResponseJson(TypedDict):
            type: str
            code: str
            detail: str
        
        class MalformedResponseJson(GenericResponseJson):
            type: Literal["validation_error"]
            attr: str

        ResponseJson = Union[GenericResponseJson, MalformedResponseJson]

        def __init__(self, *args, json: Optional[ResponseJson], **kwargs):
            super().__init__(*args, **kwargs)
            self.json = json
