import os
import uuid
import requests
import sys
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


def wait_for_merge_success(change_set_id,
                           timeout_seconds=300,
                           poll_interval=10):
    """Waits until all actions are 'Success' or the change set is 'Applied' with no actions."""
    start_time = time.time()
    while time.time() - start_time < timeout_seconds:
        response = requests.get(
            f'{API_URL}/v1/w/{WORKSPACE_ID}/change-sets/{change_set_id}/merge_status',
            headers=headers)
        response.raise_for_status()
        merge_data = response.json()

        change_set = merge_data.get("changeSet", {})
        actions = merge_data.get("actions", [])

        if not actions:
            status = change_set.get("status")
            if status == "Applied":
                print("✅ Change set applied with no actions.")
                return True
            else:
                print(
                    f"⏳ No actions found. Change set status: {status}. Waiting..."
                )

        else:
            states = [action["state"] for action in actions]
            if all(state == "Success" for state in states):
                print("✅ All actions succeeded.")
                return True
            else:
                print(f"⏳ Action states: {states}. Waiting...")

        time.sleep(poll_interval)

    raise TimeoutError(
        f"❌ Timeout reached. Merge not successful for ChangeSet {change_set_id}."
    )


def get_public_ip(change_set_id,
                  component_id,
                  timeout_seconds=60,
                  poll_interval=3):
    url = f"{API_URL}/v1/w/{WORKSPACE_ID}/change-sets/{change_set_id}/components/{component_id}"

    start_time = time.time()
    while time.time() - start_time < timeout_seconds:
        response = requests.get(url, headers=headers)
        response.raise_for_status()
        component = response.json().get("component", {})

        for prop in component.get("resourceProps", []):
            if prop.get("path") == "root/resource_value/PublicIp":
                public_ip = prop.get("value")
                if public_ip:
                    print(f"✅ Public IP found: {public_ip}")
                    return public_ip

        print("⏳ Public IP not ready yet, retrying...")
        time.sleep(poll_interval)

    raise TimeoutError("❌ Public IP not found within timeout window.")


def manage_component(change_set_id, component_id, manager_component_id):
    response = requests.post(
        f'{API_URL}/v1/w/{WORKSPACE_ID}/change-sets/{change_set_id}/components/{component_id}/manage',
        headers=headers,
        json={"componentId": manager_component_id})
    response.raise_for_status()
    return response.json()


def create_change_set(name):
    response = requests.post(f'{API_URL}/v1/w/{WORKSPACE_ID}/change-sets',
                             headers=headers,
                             json={'changeSetName': name})
    response.raise_for_status()
    return response.json()


def create_component(change_set_id, schema_name, name, options=None):
    request_body = {'schemaName': schema_name, 'name': name}
    if options:
        request_body.update(options)
    response = requests.post(
        f'{API_URL}/v1/w/{WORKSPACE_ID}/change-sets/{change_set_id}/components',
        headers=headers,
        json=request_body)
    response.raise_for_status()
    return response.json()


MANAGER_COMPONENT_ID = "01JY7K7ZBMPHG22RVTNSA6GB0Z"


def main():
    try:
        print('Starting System Initiative Environment Setup')

        branch_name = "main"
        environment_uuid = uuid.uuid4()
        change_set_name = f"Environment {environment_uuid}"
        print(f'Creating change set: {change_set_name}')
        change_set_data = create_change_set(change_set_name)
        change_set_id = change_set_data["changeSet"]["id"]
        print(f'Created ChangeSet ID: {change_set_id}')

        with open('provision.sh', 'r') as f:
            userdata_template = f.read()
        userdata_script = userdata_template.replace('{{BRANCH}}', branch_name)

        userdata_options = {
            "domain": {
                "userdataContent": userdata_script
            },
            "viewName": "Environments",
        }

        print('Creating Userdata component...')
        userdata_data = create_component(change_set_id, "Userdata",
                                         f'userdata-{str(environment_uuid)}',
                                         userdata_options)
        userdata_component_id = userdata_data["component"]["id"]
        print(f'Userdata component ID: {userdata_component_id}')

        print(
            f'Setting manager for Userdata component {userdata_component_id}...'
        )
        manage_component(change_set_id, userdata_component_id,
                         MANAGER_COMPONENT_ID)
        print('Userdata component now managed.')

    except requests.exceptions.HTTPError as err:
        print(f'HTTP Error: {err}')
        print(f'Response: {err.response.text}')
    except Exception as err:
        print(f'General Error: {err}')

    ec2_properties = {
        "InstanceType":
        "c6i.16xlarge",
        "BlockDeviceMappings": [{
            "DeviceName": "/dev/sda1",
            "Ebs": {
                "DeleteOnTermination": True,
                "VolumeSize": 100,
                "VolumeType": "gp3"
            }
        }],
        "Tags": [{
            "Key": "Name",
            "Value": "frontend-ci-validation-test-machine"
        }]
    }

    ec2_options = {
        "domain": ec2_properties,
        "subscriptions": {
            "/domain/SecurityGroupIds/0": {
                "component": "frontend-ci-validation-sg",
                "propPath": "/resource_value/GroupId",
            },
            "/domain/ImageId": {
                "component": "Arch Linux",
                "propPath": "/domain/ImageId",
            },
            "/domain/SubnetId": {
                "component": "frontend-ci-validation-subnet-pub-1",
                "propPath": "/resource_value/SubnetId",
            },
            "/domain/KeyName": {
                "component": "frontend-ci-validation-kp",
                "propPath": "/domain/KeyName",
            },
            "/domain/extra/Region": {
                "component": "us-east-1",
                "propPath": "/domain/region"
            },
            "/domain/UserData": {
                "component": f'userdata-{str(environment_uuid)}',
                "propPath": "/domain/userdataContentBase64"
            },
            "/domain/IamInstanceProfile": {
                "component": "ci-validation-instance-instance-profile",
                "propPath": "/domain/InstanceProfileName"
            },
            "/secrets/AWS Credential": {
                "component": "si-tools-sandbox",
                "propPath": "/secrets/AWS Credential"
            }
        },
        "viewName": "Environments",
    }

    print("Creating EC2 instance component...")
    ec2_data = create_component(  # Super annoying it doesn't tell you what a misaligned prop mapping is
        change_set_id,  # would be so much better if it returned something like the valid schema for the
        "AWS::EC2::Instance",  # attempted connection. It also breaks copy and paste of the component
        str(environment_uuid),
        ec2_options)

    ec2_component_id = ec2_data["component"]["id"]
    print(f'EC2 component ID: {ec2_component_id}')

    print(f'Setting manager for EC2 component {ec2_component_id}...')
    manage_component(change_set_id, ec2_component_id, MANAGER_COMPONENT_ID)
    print('EC2 component now managed.')

    print("Waiting for DVU")
    time.sleep(
        30
    )  # I really need a method here to detect DVU is complete more elegantly

    print(f'Force applying change set {change_set_id}...')
    force_apply_url = f'{API_URL}/v1/w/{WORKSPACE_ID}/change-sets/{change_set_id}/force_apply'
    response = requests.post(force_apply_url,
                             headers={
                                 'Authorization': f'Bearer {API_TOKEN}',
                                 'accept': 'application/json'
                             },
                             data='')
    response.raise_for_status()
    print('Change set applied successfully.')

    print("Waiting for actions to complete...")
    wait_for_merge_success(change_set_id)
    print("All actions completed successfully...")

    base_change_set_id = "head"
    print("Querying for public IP...")
    ip_output_file = './ip'

    try:
        public_ip = get_public_ip(base_change_set_id, ec2_component_id, 60, 5)
        print(f"Instance is reachable at: {public_ip}")
        with open(ip_output_file, 'w') as f:
            f.write(f'{public_ip}')
    except TimeoutError as e:
        print(str(e))
        sys.exit(1)


if __name__ == '__main__':
    main()
