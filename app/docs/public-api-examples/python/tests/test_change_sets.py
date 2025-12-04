"""Tests for Change Sets API"""
import os
import time
import pytest
from dotenv import load_dotenv
from system_initiative_api_client import ApiClient, Configuration
from system_initiative_api_client.api.change_sets_api import ChangeSetsApi
from system_initiative_api_client.models.create_change_set_v1_request import CreateChangeSetV1Request

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
def change_sets_api(api_config):
    """Create ChangeSetsApi instance"""
    api_client = ApiClient(api_config["config"])
    api_token = os.environ.get("SI_API_TOKEN")
    api_client.default_headers['Authorization'] = f"Bearer {api_token}"
    return ChangeSetsApi(api_client)


@pytest.fixture(scope="module")
def workspace_id(api_config):
    """Get workspace ID from config"""
    return api_config["workspace_id"]


def test_create_change_set(change_sets_api, workspace_id):
    """Test creating a new change set"""
    change_set_name = f"test-changeset-{int(time.time() * 1000)}"
    request = CreateChangeSetV1Request(change_set_name=change_set_name)

    response = change_sets_api.create_change_set(
        workspace_id=workspace_id,
        create_change_set_v1_request=request,
    )

    assert response is not None
    assert response.change_set is not None
    assert response.change_set.name == change_set_name
    assert response.change_set.id is not None

    print(f"Created change set: id={response.change_set.id}, name={response.change_set.name}, status={response.change_set.status}")


def test_list_change_sets(change_sets_api, workspace_id):
    """Test listing all change sets"""
    response = change_sets_api.list_change_sets(workspace_id=workspace_id)

    assert response is not None
    assert response.change_sets is not None
    assert isinstance(response.change_sets, list)

    print(f"Found {len(response.change_sets)} change set(s)")


def test_apply_change_set(change_sets_api, workspace_id):
    """Test applying a change set"""
    # Create a change set to apply
    change_set_name = f"test-apply-{int(time.time() * 1000)}"
    request = CreateChangeSetV1Request(change_set_name=change_set_name)

    create_response = change_sets_api.create_change_set(
        workspace_id=workspace_id,
        create_change_set_v1_request=request,
    )

    assert create_response is not None
    assert create_response.change_set is not None
    change_set_id = create_response.change_set.id

    print(f"Created change set to apply: id={change_set_id}, name={change_set_name}")

    # Apply the change set
    response = change_sets_api.force_apply(
        workspace_id=workspace_id,
        change_set_id=change_set_id,
    )

    assert response is not None
    assert response.success is True

    print(f"Applied change set: {change_set_id}")


def test_abandon_change_set(change_sets_api, workspace_id):
    """Test abandoning a change set"""
    # Create a change set to abandon
    change_set_name = f"test-abandon-{int(time.time() * 1000)}"
    request = CreateChangeSetV1Request(change_set_name=change_set_name)

    create_response = change_sets_api.create_change_set(
        workspace_id=workspace_id,
        create_change_set_v1_request=request,
    )

    assert create_response is not None
    assert create_response.change_set is not None
    change_set_id = create_response.change_set.id

    print(f"Created change set to abandon: id={change_set_id}, name={change_set_name}")

    # Abandon the change set
    response = change_sets_api.abandon_change_set(
        workspace_id=workspace_id,
        change_set_id=change_set_id,
    )

    assert response is not None
    assert response.success is True

    print(f"Abandoned change set: {change_set_id}")
