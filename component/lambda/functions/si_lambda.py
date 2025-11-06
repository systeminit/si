# Helpers for SI lambdas

from abc import abstractmethod
import boto3
import json
import os
from datetime import datetime, timezone
from typing import NotRequired, Optional, TypedDict, cast, overload
from si_redshift import Redshift
from si_lago_api import LagoApi
from si_auth_api import SiAuthApi
from si_posthog_api import PosthogApi
from si_types import WorkspaceId, timestamp_to_sql_timestamp
import logging

class SiLambdaEnv(TypedDict):
    """Log level for the logger. Defaults to INFO"""
    SI_LOG_LEVEL: NotRequired[str]
    """Enable to make this a dry-run that doesn't actually write anything anywhere"""
    SI_DRY_RUN: NotRequired[bool]

    """Secret ID or ARN to a Lambda containing authentication information for Redshift"""
    REDSHIFT_LAMBDA_ACCESS: NotRequired[str]
    """Workgroup to connect to. Defaults to platform-app-datastore"""
    REDSHIFT_LAMBDA_WORKGROUP_NAME: NotRequired[str]
    """Database to connect to. Defaults to data"""
    REDSHIFT_LAMBDA_DATABASE: NotRequired[str]

    """Lago API URL. Defaults to https://api.getlago.com/api"""
    LAGO_API_URL: NotRequired[str]
    """The Lago API token"""
    LAGO_API_TOKEN: NotRequired[str]
    """ARN to an AWS secret containing the Lago API token"""
    LAGO_API_TOKEN_ARN: NotRequired[str]

    """Posthog API URL. Defaults to https://us.i.posthog.com"""
    POSTHOG_API_URL: NotRequired[str]
    """Posthog project API key."""
    POSTHOG_PROJECT_API_KEY: NotRequired[str]

    """Auth API URL. Defaults to https://auth-api.systeminit.com"""
    AUTH_API_URL: NotRequired[str]
    BILLING_USER_EMAIL: NotRequired[str]
    BIlLING_USER_PASSWORD_ARN: NotRequired[str]
    BILLING_USER_WORKSPACE_ID: NotRequired[str]

    # Billing metric code. Defaults to resource-hours
    SI_BILLING_METRIC_CODE: NotRequired[str]
    # Cost per resource hour. Defaults to 0.007
    SI_BILLING_COST_PER_RESOURCE_HOUR: NotRequired[float]

class SiLambda:
    @classmethod
    def lambda_handler(cls, event: SiLambdaEnv = {}, _context=None):
        lambda_instance = cls(event)
        lambda_instance.run()

    def __init__(self, event: SiLambdaEnv, session = boto3.Session()):
        self.start_timestamp = datetime.now(timezone.utc)
        self.start_timestamp_sql = timestamp_to_sql_timestamp(self.start_timestamp)
        self.session = session
        self.event = event

        """Whether this is a dry-run that doesn't actually write anything anywhere"""
        self.dry_run = event.get("SI_DRY_RUN", False)
        self.billing_metric_code = event.get("SI_BILLING_METRIC_CODE", "resource-hours")
        self.billing_cost_per_resource_hour = event.get("SI_BILLING_COST_PER_RESOURCE_HOUR", 0.007)
        self._lago = None
        self._redshift = None
        self._auth_api = None
        self._posthog_api = None
        logging.getLogger().setLevel(self.getenv("SI_LOG_LEVEL", self.getenv("LOG_LEVEL", "INFO")))
        if self.dry_run:
            logging.info("Running in dry-run mode")
        logging.info(self.start_timestamp_sql)

    @property
    def lago(self) -> LagoApi:
        """Get the Lago API for this lambda, configured from the lambda environment."""
        if self._lago is None:
            lago_api_url = self.getenv("LAGO_API_URL", "https://api.getlago.com/api")
            lago_api_token = self.getenv("LAGO_API_TOKEN")
            if lago_api_token is None:
                secret = self.getenv_secret_value("LAGO_API_TOKEN_ARN")
                assert secret is not None, "LAGO_API_TOKEN or LAGO_API_TOKEN_ARN must be set"
                lago_api_token = json.loads(secret["SecretString"])["LAGO_API_TOKEN"]
            self._lago = LagoApi(lago_api_url, lago_api_token)
        
        return self._lago

    @property
    def redshift(self):
        """Get the Redshift API client, configured from the lambda environment."""
        if self._redshift is None:
            secret = self.getenv_secret_value("LAMBDA_REDSHIFT_ACCESS")
            assert secret is not None, "LAMBDA_REDSHIFT_ACCESS must be set"
            workgroup_name = self.getenv("LAMBDA_REDSHIFT_WORKGROUP_NAME", "platform-app-datastore")
            database = self.getenv("LAMBDA_REDSHIFT_DATABASE", "data")

            self._redshift = Redshift(
                self.session,
                WorkgroupName=workgroup_name,
                Database=database,
                SecretArn=secret["ARN"],
            )

        return self._redshift

    @property
    def auth_api(self):
        """Get the Auth API client, configured from the lambda environment """
        if self._auth_api is None:
            auth_api_url = self.getenv("AUTH_API_URL", "https://auth-api.systeminit.com")
            assert auth_api_url is not None, "AUTH_API_URL must be set"

            billing_user_email = self.getenv("BILLING_USER_EMAIL")
            assert billing_user_email is not None, "BILLING_USER_EMAIL must be set"

            billing_user_password = self.getenv_secret_json("BILLING_USER_PASSWORD_ARN")
            assert billing_user_password is not None, "BILLING_USER_PASSWORD_ARN must be set"

            billing_user_workspace_id = cast(Optional[WorkspaceId], self.getenv("BILLING_USER_WORKSPACE_ID"))
            assert billing_user_workspace_id is not None, "BILLING_USER_WORKSPACE_ID must be set"

            print(billing_user_password)
            self._auth_api = SiAuthApi.login(auth_api_url, billing_user_email, billing_user_password["BILLING_USER_PASWORD"], billing_user_workspace_id)

        return self._auth_api

    @property
    def posthog(self):
        """Get the Posthog API client, configured from the lambda environment """
        if self._posthog_api is None:
            posthog_api_url = self.getenv("POSTHOG_API_URL", "https://us.i.posthog.com")
            posthog_project_api_key = self.getenv("POSTHOG_PROJECT_API_KEY")
            assert posthog_project_api_key is not None, "POSTHOG_PROJECT_API_KEY must be set"
            self._posthog_api = PosthogApi(posthog_api_url, posthog_project_api_key)

        return self._posthog_api

    @overload
    def getenv(self, key: str, default: str) -> str: ...
    @overload
    def getenv(self, key: str, default: Optional[str] = None) -> Optional[str]: ...
    def getenv(self, key: str, default: Optional[str] = None) -> Optional[str]:
        """Get an environment variable, overrideable by the event values."""
        value = self.event.get(key)
        if value is not None:
            return value
        return os.getenv(key, default)

    def getenv_secret_json(self, key: str):
        secret = self.getenv_secret_value(key)
        if secret is None:
            return None
        return json.loads(secret["SecretString"])

    def getenv_secret_value(self, key: str):
        secret_id = self.getenv(key)
        if secret_id is None:
            return None
        return self.get_secret_value(secret_id)
    
    def get_secret_value(self, secret_id: str):
        """Get a secret from an arn."""
        secretsmanager = self.session.client(service_name="secretsmanager")
        return secretsmanager.get_secret_value(SecretId=secret_id)

    @abstractmethod
    def run(self): ...