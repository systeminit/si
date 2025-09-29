import os
import uuid
import requests
import sys
import time

API_TOKEN = os.environ.get('SI_API_TOKEN')
WORKSPACE_ID = os.environ.get("SI_WORKSPACE_ID")
API_URL = "https://api.systeminit.com"

if not API_TOKEN or not WORKSPACE_ID:
    error_msg = "Missing required environment variables: SI_API_TOKEN or SI_WORKSPACE_ID"
    write_error_to_file(error_msg)
    raise ValueError(error_msg)

headers = {
    'Authorization': f'Bearer {API_TOKEN}',
    'Content-Type': 'application/json'
}


def write_error_to_file(error_message):
    """Write error message to file for workflow consumption."""
    try:
        with open('./error', 'w') as f:
            f.write(error_message)
    except Exception as e:
        print(f"Failed to write error to file: {e}")


def get_action_logs(change_set_id, func_run_id):
    """Retrieves logs for a specific function run."""
    try:
        response = requests.get(
            f'{API_URL}/v1/w/{WORKSPACE_ID}/change-sets/{change_set_id}/funcs/runs/{func_run_id}',
            headers=headers)
        response.raise_for_status()
        logs_data = response.json()

        # Check for logs in the response - they're nested under funcRun.logs.logs
        func_run = logs_data.get("funcRun", {})
        if func_run and "logs" in func_run:
            logs_obj = func_run["logs"]
            if isinstance(logs_obj, dict) and "logs" in logs_obj:
                return logs_obj["logs"]

        return []
    except Exception as e:
        print(f"❌ Failed to retrieve logs for func run {func_run_id}: {e}")
        return []


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
                failed_actions = [
                    action for action in actions if action["state"] == "Failed"
                ]

                if failed_actions:
                    print(
                        f"❌ {len(failed_actions)} action(s) failed. Outputting logs:"
                    )
                    for action in failed_actions:
                        print(
                            f"\n--- Logs for failed action: {action.get('displayName', action.get('name', 'Unknown'))} ---"
                        )
                        func_run_id = action.get("funcRunId")

                        # If no funcRunId in merge_status, get it from the detailed actions endpoint
                        if not func_run_id:
                            try:
                                action_response = requests.get(
                                    f'{API_URL}/v1/w/{WORKSPACE_ID}/change-sets/{change_set_id}/actions',
                                    headers=headers)
                                action_response.raise_for_status()
                                actions_data = action_response.json()
                                for action_detail in actions_data.get(
                                        "actions", []):
                                    if action_detail.get("id") == action.get(
                                            "id"):
                                        func_run_id = action_detail.get(
                                            "funcRunId")
                                        break
                            except Exception as e:
                                print(
                                    f"Failed to get detailed action info: {e}")

                        if func_run_id:
                            # Try to get logs from HEAD first, then fallback to change_set_id
                            logs = get_action_logs("head", func_run_id)
                            if not logs:
                                logs = get_action_logs(change_set_id,
                                                       func_run_id)

                            error_message = ""
                            if logs:
                                for log in logs:
                                    timestamp = log.get("timestamp", "")
                                    stream = log.get("stream", "")
                                    message = log.get("message", "")
                                    print(f"[{timestamp}] {stream}: {message}")

                                    # Capture error messages for the error file (for EC2 deployment context)
                                    if stream == "output" and (
                                            "error" in message.lower()
                                            or "message" in message):
                                        try:
                                            # Try to parse JSON and extract error message
                                            import json
                                            output_data = json.loads(
                                                message.split("Output: ")[1]
                                                if "Output: " in
                                                message else message)
                                            if "message" in output_data:
                                                error_message = output_data[
                                                    "message"]
                                        except:
                                            error_message = message
                            else:
                                print("No logs available for this action.")

                            # Save error details to file for E2E workflow to read
                            if error_message:
                                write_error_to_file(error_message)
                        else:
                            print("No func run ID available for this action.")
                        print("--- End of logs ---\n")
                    return False  # Exit immediately when actions fail
                else:
                    print(f"⏳ Action states: {states}. Waiting...")

        time.sleep(poll_interval)

    # Get final status and output logs for any failed actions before timeout
    try:
        response = requests.get(
            f'{API_URL}/v1/w/{WORKSPACE_ID}/change-sets/{change_set_id}/merge_status',
            headers=headers)
        response.raise_for_status()
        merge_data = response.json()
        actions = merge_data.get("actions", [])

        failed_actions = [
            action for action in actions if action["state"] == "Failed"
        ]
        if failed_actions:
            print(
                f"❌ {len(failed_actions)} action(s) failed. Outputting logs:")
            for action in failed_actions:
                print(
                    f"\n--- Logs for failed action: {action.get('displayName', 'Unknown')} ---"
                )
                func_run_id = action.get("funcRunId")
                if func_run_id:
                    logs = get_action_logs(change_set_id, func_run_id)
                    if logs:
                        for log in logs:
                            timestamp = log.get("timestamp", "")
                            stream = log.get("stream", "")
                            message = log.get("message", "")
                            print(f"[{timestamp}] {stream}: {message}")
                    else:
                        print("No logs available for this action.")
                else:
                    print("No func run ID available for this action.")
                print("--- End of logs ---\n")
    except Exception as e:
        print(f"Failed to retrieve final action status: {e}")

    # Write timeout error to file before raising
    timeout_msg = f"Action execution timeout: Merge not successful for ChangeSet {change_set_id} after {timeout_seconds}s"
    write_error_to_file(timeout_msg)
    raise TimeoutError(f"❌ {timeout_msg}")


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

    ip_timeout_msg = f"Public IP lookup timeout: Instance public IP not available after {timeout_seconds}s"
    write_error_to_file(ip_timeout_msg)
    raise TimeoutError(f"❌ {ip_timeout_msg}")


def manage_component(change_set_id, component_id, manager_component_id):
    try:
        response = requests.post(
            f'{API_URL}/v1/w/{WORKSPACE_ID}/change-sets/{change_set_id}/components/{component_id}/manage',
            headers=headers,
            json={"componentId": manager_component_id})
        response.raise_for_status()
        return response.json()
    except requests.exceptions.HTTPError as e:
        error_msg = f"HTTP error setting manager for component '{component_id}': {e}"
        if hasattr(e, 'response') and e.response:
            error_msg += f" - Response: {e.response.text}"
        raise Exception(error_msg)
    except Exception as e:
        raise Exception(
            f"Failed to set manager for component '{component_id}': {str(e)}")


def force_apply_with_retry(change_set_id,
                           timeout_seconds=120,
                           retry_interval=5):
    """Force apply change set with retry logic for if DVU roots still exist."""
    start_time = time.time()
    while time.time() - start_time < timeout_seconds:
        try:
            force_apply_url = f'{API_URL}/v1/w/{WORKSPACE_ID}/change-sets/{change_set_id}/force_apply'
            response = requests.post(force_apply_url,
                                     headers={
                                         'Authorization':
                                         f'Bearer {API_TOKEN}',
                                         'accept': 'application/json'
                                     },
                                     data='')
            response.raise_for_status()
            print('Change set applied successfully.')
            return response.json()
        except requests.exceptions.HTTPError as e:
            if e.response.status_code == 428:  # PRECONDITION_REQUIRED == DVU
                elapsed = time.time() - start_time
                remaining = timeout_seconds - elapsed
                print(
                    f'⏳ DVU Roots still present. Retrying in {retry_interval}s... ({remaining:.1f}s remaining)'
                )
                if remaining > retry_interval:
                    time.sleep(retry_interval)
                    continue
                else:
                    break
            else:
                raise e

    force_apply_timeout_msg = f"Force apply timeout: Change set apply failed after {timeout_seconds}s - DVUs still processing"
    write_error_to_file(force_apply_timeout_msg)
    raise TimeoutError(f"❌ {force_apply_timeout_msg}")


def create_change_set(name):
    try:
        response = requests.post(f'{API_URL}/v1/w/{WORKSPACE_ID}/change-sets',
                                 headers=headers,
                                 json={'changeSetName': name})
        response.raise_for_status()
        return response.json()
    except requests.exceptions.HTTPError as e:
        error_msg = f"HTTP error creating change set '{name}': {e}"
        if hasattr(e, 'response') and e.response:
            error_msg += f" - Response: {e.response.text}"
        raise Exception(error_msg)
    except Exception as e:
        raise Exception(f"Failed to create change set '{name}': {str(e)}")


def create_component(change_set_id, schema_name, name, options=None):
    request_body = {'schemaName': schema_name, 'name': name}
    if options:
        request_body.update(options)
    # print(request_body)
    try:
        response = requests.post(
            f'{API_URL}/v1/w/{WORKSPACE_ID}/change-sets/{change_set_id}/components',
            headers=headers,
            json=request_body)
        response.raise_for_status()
        # print(response.json())
        return response.json()
    except requests.exceptions.HTTPError as e:
        error_msg = f"HTTP error creating component '{name}' with schema '{schema_name}': {e}"
        if hasattr(e, 'response') and e.response:
            error_msg += f" - Response: {e.response.text}"
        raise Exception(error_msg)
    except Exception as e:
        raise Exception(f"Failed to create component '{name}': {str(e)}")


def get_change_set(change_set_id):
    response = requests.get(
        f'{API_URL}/v1/w/{WORKSPACE_ID}/change-sets/{change_set_id}',
        headers=headers)
    return response


def delete_change_set(change_set_id):
    response = requests.delete(
        f'{API_URL}/v1/w/{WORKSPACE_ID}/change-sets/{change_set_id}',
        headers=headers)
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
        try:
            change_set_data = create_change_set(change_set_name)
            change_set_id = change_set_data["changeSet"]["id"]
            print(f'Created ChangeSet ID: {change_set_id}')
        except Exception as e:
            error_msg = f"Failed to create change set '{change_set_name}': {str(e)}"
            write_error_to_file(error_msg)
            print(error_msg)
            sys.exit(1)

        try:
            with open('provision.sh', 'r') as f:
                userdata_template = f.read()
        except FileNotFoundError:
            error_msg = "Deployment setup error: provision.sh script not found"
            write_error_to_file(error_msg)
            print(error_msg)
            sys.exit(1)
        except Exception as e:
            error_msg = f"Failed to read provision.sh script: {str(e)}"
            write_error_to_file(error_msg)
            print(error_msg)
            sys.exit(1)
        userdata_script = userdata_template.replace('{{BRANCH}}', branch_name)

        userdata_options = {
            "attributes": {
                "/domain/userdataContent": userdata_script
            },
            "viewName": "Environments",
        }

        print('Creating Userdata component...')
        try:
            userdata_data = create_component(
                change_set_id, "Userdata", f'userdata-{str(environment_uuid)}',
                userdata_options)
            userdata_component_id = userdata_data["component"]["id"]
            print(f'Userdata component ID: {userdata_component_id}')
        except Exception as e:
            error_msg = f"Failed to create Userdata component: {str(e)}"
            write_error_to_file(error_msg)
            print(error_msg)
            sys.exit(1)

        print(
            f'Setting manager for Userdata component {userdata_component_id}...'
        )
        try:
            manage_component(change_set_id, userdata_component_id,
                             MANAGER_COMPONENT_ID)
            print('Userdata component now managed.')
        except Exception as e:
            error_msg = f"Failed to set manager for Userdata component: {str(e)}"
            write_error_to_file(error_msg)
            print(error_msg)
            sys.exit(1)

        ec2_options = {
            "attributes": {
                "/domain/InstanceType": "c6i.16xlarge",
                "/domain/BlockDeviceMappings/0": {
                    "DeviceName": "/dev/sda1",
                    "Ebs": {
                        "DeleteOnTermination": True,
                        "VolumeSize": 100,
                        "VolumeType": "gp3"
                    }
                },
                "/domain/Tags/0": {
                    "Key": "Name",
                    "Value": "frontend-ci-validation-test-machine"
                },
                "/domain/SecurityGroupIds/0": {
                    "$source": {
                        "component": "frontend-ci-validation-sg",
                        "path": "/resource_value/GroupId",
                    }
                },
                "/domain/ImageId": {
                    "$source": {
                        "component": "Arch Linux",
                        "path": "/domain/ImageId",
                    }
                },
                "/domain/SubnetId": {
                    "$source": {
                        "component": "frontend-ci-validation-subnet-pub-1",
                        "path": "/resource_value/SubnetId",
                    }
                },
                "/domain/KeyName": {
                    "$source": {
                        "component": "frontend-ci-validation-kp",
                        "path": "/domain/KeyName",
                    }
                },
                "/domain/extra/Region": {
                    "$source": {
                        "component": "us-east-1",
                        "path": "/domain/region"
                    }
                },
                "/domain/UserData": {
                    "$source": {
                        "component": f'userdata-{str(environment_uuid)}',
                        "path": "/domain/userdataContentBase64"
                    }
                },
                "/domain/IamInstanceProfile": {
                    "$source": {
                        "component": "ci-validation-instance-instance-profile",
                        "path": "/domain/InstanceProfileName"
                    }
                },
                "/secrets/AWS Credential": {
                    "$source": {
                        "component": "si-tools-sandbox",
                        "path": "/secrets/AWS Credential"
                    }
                }
            },
            "viewName": "Environments",
        }

        print("Creating EC2 instance component...")
        try:
            ec2_data = create_component(  # Super annoying it doesn't tell you what a misaligned prop mapping is
                change_set_id,  # would be so much better if it returned something like the valid schema for the
                "AWS::EC2::Instance",  # attempted connection. It also breaks copy and paste of the component
                str(environment_uuid),
                ec2_options)

            ec2_component_id = ec2_data["component"]["id"]
            print(f'EC2 component ID: {ec2_component_id}')
        except Exception as e:
            error_msg = f"Failed to create EC2 instance component: {str(e)}"
            write_error_to_file(error_msg)
            print(error_msg)
            sys.exit(1)

        print(f'Setting manager for EC2 component {ec2_component_id}...')
        try:
            manage_component(change_set_id, ec2_component_id,
                             MANAGER_COMPONENT_ID)
            print('EC2 component now managed.')
        except Exception as e:
            error_msg = f"Failed to set manager for EC2 component: {str(e)}"
            write_error_to_file(error_msg)
            print(error_msg)
            sys.exit(1)

        print(f'Force applying change set {change_set_id}...')
        try:
            force_apply_with_retry(change_set_id)
        except Exception as e:
            error_msg = f"Failed to apply change set: {str(e)}"
            write_error_to_file(error_msg)
            print(error_msg)
            sys.exit(1)

        print("Waiting for actions to complete...")
        success = wait_for_merge_success(change_set_id)

        if not success:
            # Error should already be written by wait_for_merge_success, but ensure we have one
            if not os.path.exists('./error'):
                write_error_to_file(
                    "Deployment actions failed - check logs for details")
            print("❌ Actions failed. Exiting.")
            sys.exit(1)

        print("All actions completed successfully...")

        base_change_set_id = "head"
        print("Querying for public IP...")
        ip_output_file = './ip'

        try:
            public_ip = get_public_ip(base_change_set_id, ec2_component_id,
                                      120, 5)
            print(f"Instance is reachable at: {public_ip}")
            with open(ip_output_file, 'w') as f:
                f.write(f'{public_ip}')
        except Exception as e:
            error_msg = f"Failed to retrieve or save public IP: {str(e)}"
            write_error_to_file(error_msg)
            print(error_msg)
            sys.exit(1)

    except TimeoutError as e:
        error_msg = f"Deployment timeout: {str(e)}"
        write_error_to_file(error_msg)
        print(error_msg)
        sys.exit(1)
    except requests.exceptions.HTTPError as err:
        error_msg = f"HTTP Error during deployment: {err}"
        if hasattr(err, 'response') and err.response:
            error_msg += f" - Response: {err.response.text}"
        write_error_to_file(error_msg)
        print(f'HTTP Error: {err}')
        print(f'Response: {err.response.text}')
        sys.exit(1)
    except Exception as err:
        error_msg = f"Unexpected deployment error: {str(err)}"
        write_error_to_file(error_msg)
        print(f'General Error: {err}')
        sys.exit(1)
    finally:
        if change_set_id:
            try:
                response = get_change_set(change_set_id)
                if response.status_code == 200:
                    print(f'Cleaning up change set {change_set_id}...')
                    delete_change_set(change_set_id)
                    print('Change set deleted.')
            except Exception as cleanup_err:
                print(f'Failed to cleanup change set: {cleanup_err}')


if __name__ == '__main__':
    main()
