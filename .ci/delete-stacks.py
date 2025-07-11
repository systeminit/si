import os
import requests
import time

API_TOKEN = os.environ.get('SI_API_TOKEN')
WORKSPACE_ID = os.environ.get("SI_WORKSPACE_ID")
API_URL = "https://api.systeminit.com"

if not API_TOKEN or not WORKSPACE_ID:
    raise ValueError(
        "Missing SI_API_TOKEN or SI_WORKSPACE_ID environment variables.")

headers = {
    'Authorization': f'Bearer {API_TOKEN}',
    'Content-Type': 'application/json'
}


def create_change_set(name: str):
    response = requests.post(f"{API_URL}/v1/w/{WORKSPACE_ID}/change-sets",
                             headers=headers,
                             json={"changeSetName": name})
    response.raise_for_status()
    return response.json()["changeSet"]["id"]


def search_ec2_components(change_set_id: str):
    response = requests.post(
        f"{API_URL}/v1/w/{WORKSPACE_ID}/change-sets/{change_set_id}/components/search",
        headers=headers,
        json={"schemaName": "AWS::EC2::Instance"})
    response.raise_for_status()
    return response.json()["components"]  # List of component IDs


def delete_component(change_set_id: str, component_id: str):
    response = requests.delete(
        f"{API_URL}/v1/w/{WORKSPACE_ID}/change-sets/{change_set_id}/components/{component_id}",
        headers=headers)
    response.raise_for_status()
    return response.json()


def force_apply_change_set(change_set_id: str):
    response = requests.post(
        f"{API_URL}/v1/w/{WORKSPACE_ID}/change-sets/{change_set_id}/force_apply",
        headers=headers,
        data="")
    response.raise_for_status()
    return response.json()


def main():
    print("🔍 Creating a change set for deletion...")
    change_set_id = create_change_set("Delete All EC2 Components")
    print(f"✅ Change set created: {change_set_id}")

    print("🔍 Searching for EC2 instance components...")
    ec2_component_ids = search_ec2_components(change_set_id)

    if not ec2_component_ids:
        print("✅ No EC2 components found. Nothing to delete.")
        return

    print(f"⚠️ Found {len(ec2_component_ids)} EC2 components. Deleting...")
    for component_id in ec2_component_ids:
        print(f"🗑️ Deleting component: {component_id}")
        delete_component(change_set_id, component_id)

    print("Waiting for DVU")
    time.sleep(
        30
    )  # I really need a method here to detect DVU is complete more elegantly

    print("🚀 Applying change set to execute deletions...")
    force_apply_change_set(change_set_id)
    print("✅ All EC2 components scheduled for deletion.")


if __name__ == "__main__":
    main()
