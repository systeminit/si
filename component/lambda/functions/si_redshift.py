from collections.abc import Callable
import os
from typing import (
    Any,
    Generator,
    Literal,
    TypeVar,
    Unpack,
    NotRequired,
    TypedDict,
    overload,
)
import boto3
from mypy_boto3_redshift_data import RedshiftDataAPIServiceClient
import botocore
import botocore.session as bc
from botocore.client import Config
import time
from si_logger import logger
import logging


class DatabaseConnectParams(TypedDict):
    ClientToken: NotRequired[str]
    ClusterIdentifier: NotRequired[str]
    Database: NotRequired[str]
    DbUser: NotRequired[str]
    SecretArn: NotRequired[str]
    WorkgroupName: NotRequired[str]


FieldValue = int | str | float | bool | None

T = TypeVar("T")


class Redshift:
    @classmethod
    def from_env(
        cls,
        session=boto3.Session(),
        secret_id=os.environ["LAMBDA_REDSHIFT_ACCESS"],
        WorkgroupName="platform-app-datastore",
        Database="data",
    ):
        secretsmanager = session.client(service_name="secretsmanager")
        secret_arn = secretsmanager.get_secret_value(SecretId=secret_id)["ARN"]
        return cls(
            session,
            WorkgroupName=WorkgroupName,
            Database=Database,
            SecretArn=secret_arn,
        )

    def __init__(
        self,
        session: boto3.Session,
        *,
        wait_interval_seconds: float = 0.25,
        report_interval_seconds: float = 5,
        **database_params: Unpack[DatabaseConnectParams],
    ):
        self._session = session
        self._database_params = database_params
        self._wait_interval_seconds = wait_interval_seconds
        self._report_interval_seconds = report_interval_seconds
        self._client = self._connect()

    def query(self, Sql: str, **Parameters: object):
        return self._query(Sql, with_headers=True, **Parameters)

    def query_raw(self, Sql: str, **Parameters: object):
        return self._query(Sql, with_headers=False, **Parameters)

    def execute(self, Sql: str, **Parameters: object):
        """
        Executes a SQL statement, waiting for completion.

        Returns only when the query status is "FINISHED".

        Returns: the describe_statement response.

        Raises:
            Exception if the query failed or was aborted.
        """
        query_args = {}
        if len(Parameters) > 0:
            query_args = {
                "Parameters": [
                    {"name": name, "value": value}
                    for [name, value] in Parameters.items()
                ]
            }

        statement = self.with_client(
            lambda client: client.execute_statement(
                Sql=Sql, **self._database_params, **query_args  # type: ignore
            )
        )

        last_report = time.time()

        while True:
            response = self.with_client(
                lambda client: client.describe_statement(Id=statement["Id"])
            )
            status = response["Status"]

            match status:
                case "FINISHED":
                    return response
                case "FAILED":
                    raise Exception(
                        f"Query failed: {response['Error']} (Id={statement['Id']})"
                    )
                case "ABORTED":
                    raise Exception(f"Query aborted (Id={statement['Id']})")

            if time.time() - last_report >= self._report_interval_seconds:
                last_report = time.time()
                logger.log(logging.INFO,
                    f"Query status: {status}. Waiting {self._wait_interval_seconds}s for completion... (Id={statement['Id']})"
                )

            time.sleep(self._wait_interval_seconds)

    @overload
    def _query(
        self, Sql: str, with_headers: Literal[False], **Parameters: object
    ) -> Generator[list[FieldValue], Any, None]: ...

    @overload
    def _query(
        self, Sql: str, with_headers: Literal[True], **Parameters: object
    ) -> Generator[dict[str, FieldValue], Any, None]: ...

    def _query(self, Sql: str, with_headers: bool, **Parameters: object):
        def to_python_value(value) -> FieldValue:
            assert len(value) == 1
            return None if value.get("isNull") == True else list(value.values())[0]

        if len(Parameters) > 0:
            logger.info(f"Executing query: {Sql} with parameters {Parameters}")
            statement = self.execute(Sql, **Parameters)
        else:
            logger.info(f"Executing query: {Sql}")
            statement = self.execute(Sql)

        # Get the first page of results
        result = self.with_client(
            lambda client: client.get_statement_result(
                Id=statement["Id"],
            )
        )

        # Page through the results, yielding each one
        column_names = [col["name"] for col in result["ColumnMetadata"]]  # type: ignore
        while True:
            logger.debug(f"Page with {len(result['Records'])} records")
            for record in result["Records"]:
                values = [to_python_value(value) for value in record]
                yield dict(zip(column_names, values)) if with_headers else values

            # If this was the last page, exit.
            if "NextToken" not in result:
                break

            next_token = result["NextToken"]

            logger.info(f"Page complete. Getting next page with token {next_token} ...")
            result = self.with_client(
                lambda client: client.get_statement_result(
                    Id=statement["Id"], NextToken=next_token
                )
            )

    def with_client(self, callback: Callable[["RedshiftDataAPIServiceClient"], T]) -> T:
        """
        Calls the callback with the Redshift client, reconnecting and retrying if the
        connection has been lost.
        """

        try:
            return callback(self._client)
        except botocore.exceptions.ConnectionError as e:  # type: ignore
            logger.info(
                "Connection error! Reestablishing connection and re-executing ..."
            )
            self._client = self._connect()
            return callback(self._client)
        except Exception as e:
            raise Exception(e)

    def _connect(self):
        return self._session.client(
            "redshift-data", config=Config(connect_timeout=5, read_timeout=5)
        )
