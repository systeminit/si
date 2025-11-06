from collections.abc import Callable
from typing import (
    TypeVar,
    Unpack,
    NotRequired,
    TypedDict,
    cast,
)
import boto3

from typing import TYPE_CHECKING
if TYPE_CHECKING:
    from mypy_boto3_redshift_data import RedshiftDataAPIServiceClient
    from mypy_boto3_redshift_data.type_defs import ExecuteStatementOutputTypeDef, FieldTypeDef
import botocore
from botocore.client import Config
import time
import logging

class DatabaseConnectParams(TypedDict):
    ClientToken: NotRequired[str]
    ClusterIdentifier: NotRequired[str]
    Database: NotRequired[str]
    DbUser: NotRequired[str]
    SecretArn: NotRequired[str]
    WorkgroupName: NotRequired[str]


FieldValue = bytes | int | str | float | bool | None

T = TypeVar("T")


class Redshift:
    def __init__(
        self,
        session: boto3.Session,
        *,
        wait_interval_seconds: float = 0.25,
        report_interval_seconds: float = 10,
        **database_params: Unpack[DatabaseConnectParams],
    ):
        self._session = session
        self._database_params = database_params
        self._wait_interval_seconds = wait_interval_seconds
        self._report_interval_seconds = report_interval_seconds
        self._client = self._connect()

    def query(self, Sql: str, **Parameters: object):
        return self.QueryWithHeaders(self, self._execute_statement(Sql, **Parameters))

    def query_raw(self, Sql: str, **Parameters: object):
        return self.Query(self, self._execute_statement(Sql, **Parameters))

    def execute(self, Sql: str, **Parameters: object):
        """
        Executes a SQL statement, waiting for completion.

        Returns only when the query status is "FINISHED".

        Returns: the describe_statement response.

        Raises:
            Exception if the query failed or was aborted.
        """

        return self.start_executing(Sql, **Parameters).wait_for_complete()

    def start_executing(self, Sql: str, **Parameters: object):
        return self.Statement(self, self._execute_statement(Sql, **Parameters))

    def _execute_statement(self, Sql: str, **Parameters: object):
        if len(Parameters) > 0:
            logging.info(f"Executing SQL: {Sql} with parameters {Parameters}")
            query_args = {
                "Parameters": [
                    {"name": name, "value": value}
                    for [name, value] in Parameters.items()
                ]
            }
        else:
            logging.info(f"Executing query: {Sql}")
            query_args = {}

        return self.with_client(
            lambda client: client.execute_statement(
                Sql=Sql, **self._database_params, **query_args  # type: ignore
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
            logging.info(
                "Connection error! Reestablishing connection and re-executing ..."
            )
            self._client = self._connect()
            return callback(self._client)
        except Exception as e:
            raise Exception(e)

    def _connect(self):
        return cast("RedshiftDataAPIServiceClient", self._session.client(
            "redshift-data", config=Config(connect_timeout=5, read_timeout=5)
        ))

    class Statement:
        def __init__(self, redshift: 'Redshift', statement: 'ExecuteStatementOutputTypeDef'):
            self.redshift = redshift
            self.statement = statement
            self.response = None
            self.started_at = time.time()
            self.last_report = None

        def wait_for_complete(self):
            if self.response is not None:
                return self.response

            def log_status(status: str):
                logging.log(logging.INFO,
                    f"Query status: {status}. (Id={self.statement['Id']}, Elapsed={round(time.time() - self.started_at, 3)}s)"
                )
                self.last_report = time.time()

            while True:
                response = self._describe_statement()
                status = response["Status"]

                match status:
                    case "FINISHED":
                        log_status(status)
                        self.response = response
                        return response
                    case "FAILED":
                        log_status(status)
                        self.response = response
                        raise Exception(
                            f"Query failed: {response['Error']} (Id={self.statement['Id']})"
                        )
                    case "ABORTED":
                        log_status(status)
                        self.response = response
                        raise Exception(f"Query aborted (Id={self.statement['Id']})")

                if time.time() - (self.last_report or self.started_at) >= self.redshift._report_interval_seconds:
                    log_status(status)

                time.sleep(self.redshift._wait_interval_seconds)

        def _describe_statement(self):
            return self.redshift.with_client(
                lambda client: client.describe_statement(Id=self.statement["Id"])
            )

    class Query(Statement):
        def __iter__(self):
            for page in self._each_page():
                for record in page["Records"]:
                    yield [self._to_python_value(value) for value in record]

        def _each_page(self):
            self.wait_for_complete()

            # Get the first page of results
            result = self._get_statement_result()
            yield result

            # Get other pages of results
            while 'NextToken' in result:
                next_token = result['NextToken']

                logging.info(f"Page complete. Getting next page with token {next_token} ...")
                result = self._get_statement_result(NextToken=next_token)

                yield result

        def _get_statement_result(self, **Parameters):
            return self.redshift.with_client(
                lambda client: client.get_statement_result(
                    Id=self.statement["Id"],
                    **Parameters
                )
            )

        def _to_python_value(self, value: 'FieldTypeDef') -> FieldValue:
            assert(len(value) == 1)
            if 'blobValue' in value: return value['blobValue']
            if 'booleanValue' in value: return value['booleanValue']
            if 'doubleValue' in value: return value['doubleValue']
            if 'isNull' in value: return None
            if 'longValue' in value: return value['longValue']
            assert 'stringValue' in value
            return value['stringValue']


    class QueryWithHeaders(Query):
        def __iter__(self):
            for page in self._each_page():
                column_names = [col["name"] for col in page["ColumnMetadata"]]  # type: ignore
                for record in page["Records"]:
                    yield { column_names[i]: self._to_python_value(value) for i, value in enumerate(record) }
