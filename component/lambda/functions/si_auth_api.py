from typing import Literal, Optional, TypeVar, TypedDict, cast
from pip._vendor import requests
from si_types import WorkspaceId, OwnerPk, IsoTimestamp, Ulid

import urllib.parse
import logging

class SiAuthApi:
    @classmethod
    def login(cls, auth_api_url: str, email: str, password: str, workspace_id: WorkspaceId):
        response = requests.api.post(
            f"{auth_api_url}/auth/login",
            headers={"Content-Type": "application/json"},
            json={"email": email, "password": password, "workspaceId": workspace_id}
        )
        token = response.json().get("token")
        return cls(auth_api_url, token)

    def __init__(self, auth_api_url: str, token: str):
        self.auth_api_url = auth_api_url
        self.token = token

    def request(self, method: str, path: str, **kwargs):
        logging.debug(f"Auth API request: {method} {path}")
        response = requests.api.request(
            method,
            urllib.parse.urljoin(self.auth_api_url, path),
            headers={"Authorization": f"token {self.token}"},
            **kwargs,
        )
        try:
            response.raise_for_status()
        except requests.exceptions.HTTPError as e:
            raise self.HTTPError(
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
        
    # Function to query owner workspaces
    def owner_workspaces(self, workspace_id: WorkspaceId):
        result = self.get(f"/workspaces/{workspace_id}/ownerWorkspaces").json()
        return cast(WorkspaceOwnerWorkspaces, result)

    class HTTPError(Exception):
        def __init__(self, *args, json, **kwargs):
            super().__init__(*args, **kwargs)
            self.json = json

InstanceEnvType = Literal['LOCAL', 'PRIVATE', 'SI']
Role = Literal['OWNER', 'APPROVER', 'EDITOR']

class WorkspaceOwnerWorkspaces(TypedDict):
    workspaceId: WorkspaceId
    workspaceOwnerId: OwnerPk
    workspaces: list['OwnedWorkspace']

class Workspace(TypedDict):
    id: WorkspaceId
    instanceEnvType: InstanceEnvType
    instanceUrl: str
    displayName: str
    description: Optional[str]
    isDefault: bool
    isFavourite: bool
    creatorUserId: OwnerPk
    creatorUser: 'WorkspaceCreatorUser'
    token: Ulid
    deletedAt: Optional[IsoTimestamp]
    quarantinedAt: Optional[IsoTimestamp]

class OwnedWorkspace(Workspace):
    role: Role
    invitedAt: Optional[IsoTimestamp]

class WorkspaceCreatorUser(TypedDict):
    firstName: str
    lastName: str
