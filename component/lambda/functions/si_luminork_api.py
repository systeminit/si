from typing import Literal, TypedDict, cast
from pip._vendor import requests
from si_types import ChangeSetId, ComponentId, WorkspaceId

import logging
import urllib.parse

class SiLuminorkApi:
    def __init__(self, api_url: str, token: str, workspace_id: WorkspaceId, change_set_id: ChangeSetId):
        self.api_url = api_url
        self.token = token
        self.workspace_id = workspace_id
        self.change_set_id = change_set_id

    def request(self, method: str, path: str, **kwargs):
        logging.debug(f"Auth API request: {method} {path}")
        response = requests.api.request(
            method,
            urllib.parse.urljoin(self.api_url, path),
            headers={"Authorization": f"Bearer {self.token}"},
            **kwargs,
        )
        try:
            response.raise_for_status()
        except requests.exceptions.HTTPError as e:
            try:
                json = e.response.json()
            except Exception:
                json = None
            raise self.HTTPError(
                f"{e.response.status_code} for {method} {path}: {e.response.text}",
                json=json,
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
    
    def search_spike(self, attr_name: str, attr_value: str, search_method: str):
        response = self.post(f"/v1/w/{self.workspace_id}/change-sets/{self.change_set_id}/components/search_spike", {
            "attrName": attr_name,
            "attrValue": attr_value,
            "searchMethod": search_method,
        })
        return cast(SearchSpikeResults, response.json())["componentIds"]

    def get_component(self, component_id: ComponentId):
        response = self.get(f"/v1/w/{self.workspace_id}/change-sets/{self.change_set_id}/components/{component_id}")
        return response.json()

    def create_component(self, schema_name: str, component_name: str):
        response = self.post(f"/v1/w/{self.workspace_id}/change-sets/{self.change_set_id}/components", {
            "schemaName": schema_name,
            "name": component_name,
            "attributes": {}
        })
        return cast(ComponentId, response.json()["component"]["id"])

    class HTTPError(Exception):
        def __init__(self, *args, json, **kwargs):
            super().__init__(*args, **kwargs)
            self.json = json

class SearchSpikeResults(TypedDict):
    componentIds: list[ComponentId]
