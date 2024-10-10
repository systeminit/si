import os
import json
import boto3
import botocore
from pip._vendor import requests
import botocore.session as bc
from botocore.client import Config
import time
from datetime import datetime

secret_name = os.environ['LAMBDA_REDSHIFT_ACCESS']
auth_api_url = os.environ['AUTH_API_URL']
billing_user_email = os.environ['BILLING_USER_EMAIL']
billing_user_password_arn = os.environ['BILLING_USER_PASSWORD_ARN']
billing_user_workspace_id = os.environ['BILLING_USER_WORKSPACE_ID']

lago_api_url = os.environ['LAGO_API_URL']
lago_api_token_arn = os.environ['LAGO_API_TOKEN_ARN']

session = boto3.session.Session()
region = session.region_name

# Initializing Secret Manager's client    
client = session.client(
    service_name='secretsmanager',
    region_name=region
)

get_secret_value_response = client.get_secret_value(
    SecretId=secret_name
)
secret_arn = get_secret_value_response['ARN']

secret = get_secret_value_response['SecretString']
secret_json = json.loads(secret)

# Initializing Botocore client
bc_session = bc.get_session()

session = boto3.Session(
    botocore_session=bc_session,
    region_name=region
)

# Initializing Redshift's client   
config = Config(connect_timeout=5, read_timeout=5)
client_redshift = session.client("redshift-data", config=config)

# Get an authentication token for Auth API
def get_auth_api_token():
    get_secret_value_response = client.get_secret_value(SecretId=billing_user_password_arn)
    billing_user_password=json.loads(get_secret_value_response['SecretString'])['BILLING_USER_PASWORD'] 
    response = requests.post(
        f"{auth_api_url}/auth/login",
        headers={"Content-Type": "application/json"},
        json={"email": billing_user_email, "password": billing_user_password, "workspaceId": billing_user_workspace_id}
    )
    return response.json().get("token")

# Get an authentication token for Lago
def get_lago_api_token():
    get_secret_value_response = client.get_secret_value(SecretId=lago_api_token_arn)
    lago_api_token=json.loads(get_secret_value_response['SecretString'])['LAGO_API_TOKEN']
    return lago_api_token

# Helper function to wait for the query to complete
def wait_for_completion(statement_id, client_redshift):
    while True:
        describe_response = client_redshift.describe_statement(Id=statement_id)
        status = describe_response['Status']
        
        if status == 'FINISHED':
            break
        elif status == 'FAILED':
            raise Exception(f"Query failed: {describe_response['Error']}")
        else:
            time.sleep(1)
    return describe_response

# Transformation function to convert records into a list of dictionaries
def transform_records(column_metadata, records):
    column_names = [col['name'] for col in column_metadata]
    output = []
    
    for record in records:
        row_object = {
            column_names[i]: (record[i].get('stringValue') or record[i].get('longValue'))
            for i in range(len(column_names))
        }
        output.append(row_object)
    
    return output

# Function to query owner workspaces
def query_owner_workspaces(token, workspace_id):
    url = f"{auth_api_url}/workspaces/{workspace_id}/ownerWorkspaces"
    response = requests.get(url, headers={"Authorization": f"token {token}"})
    
    if response.status_code == 200:
        return response.json()
    else:
        raise Exception(f"Failed to query owner workspaces for {workspace_id}: {response.status_code} - {response.text}")


# Function to query owner workspaces
def query_owner_subscriptions(lago_api_token, workspace_owner_id):
    url = f"{lago_api_url}/v1/subscriptions?external_customer_id={workspace_owner_id}&status[]=pending&status[]=active&status[]=terminated&status[]=active"
    response = requests.get(url, headers={"Authorization": f"Bearer {lago_api_token}"})
    if response.status_code == 200:
        return response.json().get('subscriptions', [])
    else:
        raise Exception(f"Failed to query subscriptions for workspace_owner_id {workspace_owner_id}: {response.status_code} - {response.text}")

# Function to execute a SQL query and return the results
def execute_query(query):
    try:
        data_response = client_redshift.execute_statement(
            WorkgroupName='platform-app-datastore',
            Database='data',
            SecretArn=secret_arn,
            Sql=query
        )
        data_statement_id = data_response['Id']

        # Wait for the data query to complete using the helper function
        wait_for_completion(data_statement_id, client_redshift)

        # Determine if the query is a SELECT or an INSERT
        if query.strip().upper().startswith("SELECT"):
            # Retrieve the result data for SELECT queries
            data_result = client_redshift.get_statement_result(Id=data_statement_id)
            # Transform the records into the desired format
            output = transform_records(data_result['ColumnMetadata'], data_result['Records'])
            return {
                'statusCode': 200,
                'body': json.dumps(output)  # Return the formatted data
            }
        else:
            # For INSERT or other queries, return a success message
            return {
                'statusCode': 200,
                'body': json.dumps({"message": "Query executed successfully."})
            }

    except botocore.exceptions.ConnectionError as e:
        client_redshift_1 = session.client("redshift-data", config=config)
        result = client_redshift_1.execute_statement(
            WorkgroupName='platform-app-datastore', 
            Database='data', 
            SecretArn=secret_arn, 
            Sql=query  # Use the passed query
        )
        print("API executed after re-establishing the connection")
        return str(result)
        
    except Exception as e:
        raise Exception(e)

list_missing_workspaces = """
SELECT DISTINCT wue.workspace_id
FROM "workspace_update_events"."workspace_update_events" wue
LEFT JOIN "workspace_operations"."workspace_owners" wo
ON wue.workspace_id = wo.workspace_id
WHERE wo.workspace_id IS NULL LIMIT 50;
"""

# Convert ISO 8601 timestamp to the required format
def convert_iso_to_datetime(iso_str):
    return datetime.strptime(iso_str, '%Y-%m-%dT%H:%M:%SZ').strftime('%Y-%m-%d %H:%M:%S')

def create_insert_query(workspace_id, workspace_owner_id, subscription):
    lago_id = subscription['lago_id']
    subscription_name = subscription['name']
    plan_code = subscription['plan_code']
    
    # Prepare a list of columns and values
    columns = ["workspace_id", "owner_pk", "subscription_id", "subscription_name", "plan_code"]
    values = [f"'{workspace_id}'", f"'{workspace_owner_id}'", f"'{lago_id}'", f"'{subscription_name}'", f"'{plan_code}'"]

    # Handle optional timestamps
    record_timestamp = time.strftime('%Y-%m-%d %H:%M:%S')
    columns.append("record_timestamp")
    values.append(f"'{record_timestamp}'")

    # Handle started_at and ending_at
    started_at = subscription.get('started_at')
    ending_at = subscription.get('ending_at')

    if started_at:
        columns.append("subscription_start_date")
        values.append(f"'{convert_iso_to_datetime(started_at)}'")
    
    if ending_at:
        columns.append("subscription_end_date")
        values.append(f"'{convert_iso_to_datetime(ending_at)}'")

    # Construct the SQL query
    columns_str = ", ".join(columns)
    values_str = ", ".join(values)

    return f"""
    INSERT INTO "workspace_operations"."workspace_delegations" 
    ({columns_str}) 
    VALUES ({values_str});
    """

def create_insert_workspace_owner_query(workspace_id, workspace_owner_id, timestamp):
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

    return f"""
    INSERT INTO "workspace_operations"."workspace_owners" 
    ({columns_str}) 
    VALUES ({values_str});
    """

def create_insert_workspace_owner_subscription_query(workspace_owner_id, subscription, timestamp):
    lago_id = subscription['lago_id']
    plan_code = subscription['plan_code']
    external_id = subscription['external_id']
    
    # Prepare a list of columns and values for the workspace_owner_subscriptions table
    columns = [
        "owner_pk", 
        "subscription_id",
        "subscription_start_date", 
        "subscription_end_date", 
        "plan_code", 
        "record_timestamp",
        "external_id"
    ]
    values = [
        f"'{workspace_owner_id}'", 
        f"'{lago_id}'", 
        # Check if 'started_at' is present before converting
        f"'{convert_iso_to_datetime(subscription['started_at'])}'" if subscription.get('started_at') else 'NULL', 
        # Check if 'ending_at' is present before converting
        f"'{convert_iso_to_datetime(subscription['ending_at'])}'" if subscription.get('ending_at') else 'NULL', 
        f"'{plan_code}'", 
        f"'{timestamp}'",
        f"'{external_id}'"
    ]

    # Construct the SQL query for inserting into workspace_owner_subscriptions
    columns_str = ", ".join(columns)
    values_str = ", ".join(values)

    return f"""
    INSERT INTO "workspace_operations"."workspace_owner_subscriptions" 
    ({columns_str}) 
    VALUES ({values_str});
    """

# Function to query owner workspaces
def query_owner_subscriptions(lago_api_token, workspace_owner_id):
    url = f"{lago_api_url}/v1/subscriptions?external_customer_id={workspace_owner_id}&status[]=pending&status[]=active&status[]=terminated&status[]=active"
    response = requests.get(url, headers={"Authorization": f"Bearer {lago_api_token}"})
    if response.status_code == 200:
        return response.json().get('subscriptions', [])
    else:
        raise Exception(f"Failed to query subscriptions for workspace_owner_id {workspace_owner_id}: {response.status_code} - {response.text}")

# Function to query owner subscriptions
def query_owner_workspaces(token, workspace_id):
    url = f"{auth_api_url}/workspaces/{workspace_id}/ownerWorkspaces"
    response = requests.get(url, headers={"Authorization": f"token {token}"})

    if response.status_code == 200:
        return response.json()
    else:
        raise Exception(f"Failed to query owner workspaces for {workspace_id}: {response.status_code} - {response.text}")

def lambda_handler(event, context):

    # Get Tokens for Auth API and Lago
    auth_api_token = get_auth_api_token()
    lago_api_token = get_lago_api_token()

    # Execute the query to find missing workspaces
    missing_workspaces_result = execute_query(list_missing_workspaces)

    # Extract missing workspace IDs from the result
    missing_workspaces = json.loads(missing_workspaces_result['body'])
    workspace_ids = [workspace['workspace_id'] for workspace in missing_workspaces]

    # Prepare to collect owner workspace results
    owner_workspace_results = {}
    insert_workspace_owner_results = []
    insert_workspace_owner_subscriptions_results = []

    # Query workspace owner and owner subscriptions for each missing workspace
    for workspace_id in workspace_ids:
        print(f"Processing Workspace ID: {workspace_id}")

        # Get the current timestamp for record insertion
        current_timestamp = time.strftime('%Y-%m-%d %H:%M:%S')
        
        # Query owner workspace information
        owner_workspace_data = query_owner_workspaces(auth_api_token, workspace_id)
        workspace_owner_id = owner_workspace_data.get("workspaceOwnerId")
        owner_workspace_results[workspace_id] = owner_workspace_data

        # Query subscriptions for the workspace owner
        subscriptions = query_owner_subscriptions(lago_api_token, workspace_owner_id)

        # Insert each workspace owner and their subscriptions into the database
        print(f"Inserting workspace and owner into workspace_owners: {workspace_id} & {workspace_owner_id}")
        insert_owner_query = create_insert_workspace_owner_query(workspace_id, workspace_owner_id, current_timestamp)
        insert_owner_result = execute_query(insert_owner_query)
        insert_workspace_owner_results.append(insert_owner_result)

        # Insert each subscription into the database
        for subscription in subscriptions:
            print(f"Inserting subscription into workspace_owner_subscriptions: {subscription}")
            insert_subscription_query = create_insert_workspace_owner_subscription_query(workspace_owner_id, subscription, current_timestamp)
            insert_subscription_result = execute_query(insert_subscription_query)
            insert_workspace_owner_subscriptions_results.append(insert_subscription_result)

    return {
        'statusCode': 200,
        'body': json.dumps({
            'missing_workspaces': missing_workspaces_result,
            'owner_workspace_results': owner_workspace_results,
            'insert_workspace_owner_results': insert_workspace_owner_results,
            'insert_workspace_owner_subscriptions_results': insert_workspace_owner_subscriptions_results
        })
    }
