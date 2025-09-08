#!/usr/bin/env python3
"""
Test script for the new SDF Remote Shell API endpoint.
"""

import json
import requests
import sys
import argparse
from datetime import datetime

def test_remote_shell_api(base_url, workspace_id, change_set_id, auth_token=None):
    """Test the remote shell API endpoint."""
    
    endpoint = f"{base_url}/api/v2/workspaces/{workspace_id}/change-sets/{change_set_id}/remote-shell/create"
    
    # Test request payload
    request_data = {
        "image": "ubuntu:20.04",
        "workingDir": "/workspace", 
        "envVars": {
            "USER": "developer",
            "TEST_VAR": "test_value"
        }
    }
    
    headers = {
        "Content-Type": "application/json",
        "Accept": "application/json"
    }
    
    if auth_token:
        headers["Authorization"] = f"Bearer {auth_token}"
    
    print(f"🧪 Testing Remote Shell API")
    print(f"   Endpoint: {endpoint}")
    print(f"   Request: {json.dumps(request_data, indent=2)}")
    print()
    
    try:
        response = requests.post(endpoint, json=request_data, headers=headers, timeout=30)
        
        print(f"📡 Response Status: {response.status_code}")
        print(f"   Headers: {dict(response.headers)}")
        
        if response.headers.get('content-type', '').startswith('application/json'):
            try:
                response_data = response.json()
                print(f"   Body: {json.dumps(response_data, indent=2)}")
                
                # Validate response structure for success case
                if response.status_code == 200:
                    if validate_success_response(response_data):
                        print("✅ Success response structure is valid")
                        return True, response_data
                    else:
                        print("❌ Success response structure is invalid")
                        return False, response_data
                        
                elif response.status_code >= 400:
                    print("⚠️  API returned an error (expected if services aren't running)")
                    return False, response_data
                    
            except json.JSONDecodeError:
                print(f"   Body (raw): {response.text}")
                
        else:
            print(f"   Body (raw): {response.text}")
            
        return response.status_code == 200, None
        
    except requests.exceptions.RequestException as e:
        print(f"❌ Request failed: {e}")
        return False, None

def validate_success_response(data):
    """Validate that the success response has the expected structure."""
    
    # Check top-level structure  
    if not isinstance(data, dict):
        print("   - Response is not a dictionary")
        return False
        
    if "data" not in data:
        print("   - Missing 'data' field")
        return False
        
    if "forcedChangeSetId" not in data:
        print("   - Missing 'forcedChangeSetId' field") 
        return False
        
    session_data = data["data"]
    
    # Check required fields in session data
    required_fields = [
        "executionId", "sessionId", "containerId", 
        "connectionInfo", "status"
    ]
    
    for field in required_fields:
        if field not in session_data:
            print(f"   - Missing required field: {field}")
            return False
            
    # Check connection info structure
    connection_info = session_data["connectionInfo"]
    required_connection_fields = [
        "natsSubject", "stdinSubject", "stdoutSubject", 
        "stderrSubject", "controlSubject"
    ]
    
    for field in required_connection_fields:
        if field not in connection_info:
            print(f"   - Missing connection info field: {field}")
            return False
            
    # Check that NATS subjects follow expected pattern
    execution_id = session_data["executionId"]
    expected_prefix = f"remote_shell.{execution_id}."
    
    for subject_type in ["stdin", "stdout", "stderr", "control"]:
        subject_key = f"{subject_type}Subject"
        if subject_key in connection_info:
            subject_value = connection_info[subject_key]
            if not subject_value.startswith(expected_prefix):
                print(f"   - {subject_key} doesn't match expected pattern")
                return False
                
    return True

def test_data_structures():
    """Test that our request/response data structures are valid."""
    print("🧪 Testing data structures...")
    
    # Test request structure
    request_data = {
        "image": "ubuntu:20.04",
        "workingDir": "/workspace",
        "envVars": {"USER": "test"}
    }
    
    try:
        json.dumps(request_data)
        print("✅ Request data structure is valid JSON")
    except Exception as e:
        print(f"❌ Request data structure error: {e}")
        return False
        
    # Test mock response structure
    mock_response = {
        "forcedChangeSetId": "01234567-89ab-cdef-0123-456789abcdef",
        "data": {
            "executionId": "remote_shell_01234567890123456789012345",
            "sessionId": "session_remote_shell_01234567890123456789012345",
            "containerId": "container_remote_shell_01234567890123456789012345",
            "connectionInfo": {
                "natsSubject": "remote_shell.remote_shell_01234567890123456789012345.control",
                "stdinSubject": "remote_shell.remote_shell_01234567890123456789012345.stdin",
                "stdoutSubject": "remote_shell.remote_shell_01234567890123456789012345.stdout",
                "stderrSubject": "remote_shell.remote_shell_01234567890123456789012345.stderr",
                "controlSubject": "remote_shell.remote_shell_01234567890123456789012345.control"
            },
            "status": "Active",
            "message": "Test message"
        }
    }
    
    try:
        json.dumps(mock_response)
        if validate_success_response(mock_response):
            print("✅ Response data structure is valid")
            return True
        else:
            print("❌ Response data structure validation failed")
            return False
    except Exception as e:
        print(f"❌ Response data structure error: {e}")
        return False

def main():
    parser = argparse.ArgumentParser(description="Test SDF Remote Shell API")
    parser.add_argument(
        "--url", 
        default="http://localhost:5156", 
        help="Base URL for SDF API (default: http://localhost:5156)"
    )
    parser.add_argument(
        "--workspace",
        default="test-workspace",
        help="Workspace ID to use (default: test-workspace)"
    )
    parser.add_argument(
        "--changeset", 
        default="test-changeset",
        help="Change set ID to use (default: test-changeset)"
    )
    parser.add_argument(
        "--token",
        help="Authentication token (optional)"
    )
    parser.add_argument(
        "--data-only",
        action="store_true",
        help="Only test data structures, don't make API call"
    )
    
    args = parser.parse_args()
    
    print(f"=== SDF Remote Shell API Test ===")
    print(f"Timestamp: {datetime.now().isoformat()}")
    print()
    
    # Always test data structures first
    if not test_data_structures():
        print("\n❌ Data structure tests failed")
        return False
        
    if args.data_only:
        print("\n✅ Data structure tests passed (API test skipped)")
        return True
        
    print()
    
    # Test the actual API
    success, response_data = test_remote_shell_api(
        args.url,
        args.workspace, 
        args.changeset,
        args.token
    )
    
    print()
    if success:
        print("🎉 API test passed!")
        if response_data:
            execution_id = response_data["data"]["executionId"]
            print(f"\n📋 Session Details:")
            print(f"   Execution ID: {execution_id}")
            print(f"   Session ID: {response_data['data']['sessionId']}")
            print(f"   Status: {response_data['data']['status']}")
            
            print(f"\n📡 NATS Subjects:")
            conn_info = response_data["data"]["connectionInfo"]
            for key, value in conn_info.items():
                print(f"   {key}: {value}")
        return True
    else:
        print("❌ API test failed or services not running")
        print("\nTo test with running services:")
        print("1. Start cyclone: cargo run --bin cyclone -- --enable-remote-shell")
        print("2. Start veritech: cargo run --bin veritech")
        print("3. Start SDF: cargo run --bin sdf")
        print("4. Run this test again")
        return False

if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)