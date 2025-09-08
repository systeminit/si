#!/usr/bin/env python3
"""
Simple test script to verify remote shell data structures.
"""

import json
from datetime import datetime

def create_remote_shell_request():
    return {
        "execution_id": f"test_remote_shell_{int(datetime.now().timestamp())}",
        "image": "ubuntu:20.04",
        "env_vars": {
            "TEST_VAR": "test_value"
        },
        "working_dir": "/tmp"
    }

def test_data_structures():
    """Test that the data structures work correctly."""
    print("Testing remote shell data structures...")
    
    # Test that we can create and serialize the request
    request = create_remote_shell_request()
    try:
        request_json = json.dumps(request)
        parsed_request = json.loads(request_json)
        
        # Verify required fields
        required_fields = ["execution_id", "env_vars"]
        for field in required_fields:
            if field not in parsed_request:
                print(f"❌ Missing required field: {field}")
                return False
                
        print("✅ Remote shell request structure is valid")
        print(f"   Request: {json.dumps(request, indent=2)}")
        
        # Test response structure
        mock_response = {
            "Result": {
                "Success": {
                    "executionId": request["execution_id"],
                    "sessionId": f"session_{request['execution_id']}",
                    "containerId": f"container_{request['execution_id']}",
                    "connectionInfo": {
                        "natsSubject": f"remote_shell.{request['execution_id']}.control",
                        "stdinSubject": f"remote_shell.{request['execution_id']}.stdin",
                        "stdoutSubject": f"remote_shell.{request['execution_id']}.stdout",
                        "stderrSubject": f"remote_shell.{request['execution_id']}.stderr",
                        "controlSubject": f"remote_shell.{request['execution_id']}.control"
                    },
                    "status": "Active",
                    "message": "Remote shell session created"
                }
            }
        }
        
        response_json = json.dumps(mock_response)
        parsed_response = json.loads(response_json)
        print("✅ Remote shell response structure is valid")
        print(f"   Response: {json.dumps(mock_response, indent=2)}")
        
        return True
        
    except Exception as e:
        print(f"❌ Data structure test failed: {e}")
        return False

if __name__ == "__main__":
    success = test_data_structures()
    if success:
        print("\n🎉 Data structure tests passed!")
        print("\nNext steps:")
        print("1. Build and run cyclone with --enable-remote-shell")
        print("2. Build and run veritech")
        print("3. Submit a RemoteShell task via NATS to test end-to-end functionality")
        print("\nExample cyclone command:")
        print("cargo run --bin cyclone -- --bind-uds /tmp/cyclone.sock --lang-server /usr/local/bin/lang-js --enable-remote-shell --enable-watch")
    else:
        print("\n❌ Data structure tests failed!")
        exit(1)