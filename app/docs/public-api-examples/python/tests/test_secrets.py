"""Tests for Secrets API"""
import os
import time
import pytest
from dotenv import load_dotenv
from system_initiative_api_client import ApiClient, Configuration
from system_initiative_api_client.api.change_sets_api import ChangeSetsApi
from system_initiative_api_client.api.components_api import ComponentsApi
from system_initiative_api_client.api.secrets_api import SecretsApi
from system_initiative_api_client.models.create_change_set_v1_request import CreateChangeSetV1Request
from system_initiative_api_client.models.create_component_v1_request import CreateComponentV1Request
from system_initiative_api_client.models.create_secret_v1_request import CreateSecretV1Request
from system_initiative_api_client.models.update_secret_v1_request import UpdateSecretV1Request

# Load environment variables from .env file
load_dotenv()


@pytest.fixture(scope="module")
def api_config():
    """Create API configuration from environment variables"""
    api_token = os.environ.get("SI_API_TOKEN")
    base_path = os.environ.get("SI_API_BASE_PATH", "https://api.systeminit.com")
    workspace_id = os.environ.get("SI_WORKSPACE_ID", "")

    if not api_token:
        raise ValueError("SI_API_TOKEN environment variable is required")

    if not workspace_id:
        raise ValueError("SI_WORKSPACE_ID environment variable is required")

    config = Configuration(host=base_path)
    config.access_token = api_token

    return {
        "config": config,
        "workspace_id": workspace_id,
    }


@pytest.fixture(scope="module")
def secrets_api(api_config):
    """Create SecretsApi instance"""
    api_client = ApiClient(api_config["config"])
    api_token = os.environ.get("SI_API_TOKEN")
    api_client.default_headers['Authorization'] = f"Bearer {api_token}"
    return SecretsApi(api_client)


@pytest.fixture(scope="module")
def change_sets_api(api_config):
    """Create ChangeSetsApi instance"""
    api_client = ApiClient(api_config["config"])
    api_token = os.environ.get("SI_API_TOKEN")
    api_client.default_headers['Authorization'] = f"Bearer {api_token}"
    return ChangeSetsApi(api_client)


@pytest.fixture(scope="module")
def components_api(api_config):
    """Create ComponentsApi instance"""
    api_client = ApiClient(api_config["config"])
    api_token = os.environ.get("SI_API_TOKEN")
    api_client.default_headers['Authorization'] = f"Bearer {api_token}"
    return ComponentsApi(api_client)


@pytest.fixture(scope="module")
def workspace_id(api_config):
    """Get workspace ID from config"""
    return api_config["workspace_id"]


@pytest.fixture(scope="module")
def change_set_id(change_sets_api, workspace_id):
    """Create a change set for testing"""
    change_set_name = f"test-secrets-{int(time.time() * 1000)}"
    request = CreateChangeSetV1Request(change_set_name=change_set_name)

    response = change_sets_api.create_change_set(
        workspace_id=workspace_id,
        create_change_set_v1_request=request,
    )

    return response.change_set.id


def test_create_aws_credential_secret(secrets_api, workspace_id, change_set_id):
    """Test creating an AWS credential secret"""
    request = CreateSecretV1Request(
        name="my-aws-credentials",
        definition_name="AWS Credential",
        description="My AWS access credentials",
        raw_data={
            "accessKeyId": "AKIAIOSFODNN7EXAMPLE",
            "secretAccessKey": "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
        },
    )

    response = secrets_api.create_secret(
        workspace_id=workspace_id,
        change_set_id=change_set_id,
        create_secret_v1_request=request,
    )

    assert response is not None
    assert response.secret is not None
    assert response.secret.name == "my-aws-credentials"
    assert response.secret.definition == "AWS Credential"
    assert response.secret.description == "My AWS access credentials"


def test_create_aws_credential_secret_and_subscribe_ec2_instance(
    secrets_api, components_api, workspace_id, change_set_id
):
    """Test creating an AWS credential secret and subscribing an EC2 instance to it"""
    # Create the AWS credential secret
    secret_request = CreateSecretV1Request(
        name="test-aws-ec2-credentials",
        definition_name="AWS Credential",
        description="AWS credentials for EC2 instance",
        raw_data={
            "accessKeyId": "AKIAIOSFODNN7EXAMPLE",
            "secretAccessKey": "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
        },
    )

    secret_response = secrets_api.create_secret(
        workspace_id=workspace_id,
        change_set_id=change_set_id,
        create_secret_v1_request=secret_request,
    )

    assert secret_response is not None
    assert secret_response.secret is not None

    secret_id = secret_response.secret.id

    # Create an AWS EC2 Instance component
    component_request = CreateComponentV1Request(
        name="test-ec2-instance",
        schema_name="AWS::EC2::Instance",
        attributes={
            "/secrets/AWS Credential": {
                "component": secret_id,
                "path": "/secrets/AWS Credential",
            },
        },
    )

    component_response = components_api.create_component(
        workspace_id=workspace_id,
        change_set_id=change_set_id,
        create_component_v1_request=component_request,
    )

    assert component_response is not None
    assert component_response.component is not None


def test_update_secret_with_new_data(secrets_api, workspace_id, change_set_id):
    """Test updating a secret with new data"""
    # Create the initial secret
    create_request = CreateSecretV1Request(
        name="updatable-aws-credentials",
        definition_name="AWS Credential",
        description="Initial credentials",
        raw_data={
            "accessKeyId": "AKIAIOSFODNN7EXAMPLE",
            "secretAccessKey": "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
        },
    )

    create_response = secrets_api.create_secret(
        workspace_id=workspace_id,
        change_set_id=change_set_id,
        create_secret_v1_request=create_request,
    )

    assert create_response is not None
    assert create_response.secret is not None

    secret_id = create_response.secret.id

    # Update the secret with new data
    update_request = UpdateSecretV1Request(
        id=secret_id,
        name="updated-aws-credentials",
        description="Updated credentials with new keys",
        raw_data={
            "accessKeyId": "AKIAIOSFODNN7NEWKEY",
            "secretAccessKey": "wJalrXUtnFEMI/K7MDENG/bPxRfiCYNEWKEY",
        },
    )

    update_response = secrets_api.update_secret(
        workspace_id=workspace_id,
        change_set_id=change_set_id,
        secret_id=secret_id,
        update_secret_v1_request=update_request,
    )

    assert update_response is not None
    assert update_response.secret is not None
    assert update_response.secret.id == secret_id
    assert update_response.secret.name == "updated-aws-credentials"
    assert update_response.secret.description == "Updated credentials with new keys"


def test_delete_secret(secrets_api, workspace_id, change_set_id):
    """Test deleting a secret"""
    # Create a secret to delete
    create_request = CreateSecretV1Request(
        name="deletable-aws-credentials",
        definition_name="AWS Credential",
        description="Credentials to be deleted",
        raw_data={
            "accessKeyId": "AKIAIOSFODNN7EXAMPLE",
            "secretAccessKey": "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
        },
    )

    create_response = secrets_api.create_secret(
        workspace_id=workspace_id,
        change_set_id=change_set_id,
        create_secret_v1_request=create_request,
    )

    assert create_response is not None
    assert create_response.secret is not None

    secret_id = create_response.secret.id

    # Delete the secret
    delete_response = secrets_api.delete_secret(
        workspace_id=workspace_id,
        change_set_id=change_set_id,
        secret_id=secret_id,
    )

    assert delete_response is not None
    assert delete_response.success is True
