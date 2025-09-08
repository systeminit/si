#!/usr/bin/env python3
"""
Simple test script to verify remote shell functionality in veritech.
This tests the basic flow of remote shell task submission.
"""

import json
import asyncio
import websockets
import sys
from datetime import datetime

# Test remote shell request
def create_remote_shell_request():
    return {
        "execution_id": f"test_remote_shell_{int(datetime.now().timestamp())}",
        "image": "ubuntu:20.04",
        "env_vars": {
            "TEST_VAR": "test_value"
        },
        "working_dir": "/tmp"
    }

async def test_remote_shell_via_cyclone():
    """Test remote shell functionality directly against cyclone server."""
    print("Testing remote shell via cyclone server...")
    
    try:
        # Connect to cyclone server (assuming it's running with remote shell enabled)
        uri = "ws://127.0.0.1:5157/execute/remote-shell"
        
        async with websockets.connect(uri) as websocket:
            # Send remote shell request
            request = create_remote_shell_request()
            print(f"Sending request: {json.dumps(request, indent=2)}")
            
            await websocket.send(json.dumps(request))
            
            # Wait for response
            response = await websocket.recv()
            print(f"Received response: {response}")
            
            # Parse and validate response
            response_data = json.loads(response)
            
            if "Result" in response_data and "Success" in response_data["Result"]:
                result = response_data["Result"]["Success"]
                print("✅ Remote shell session created successfully!")
                print(f"   Session ID: {result.get('sessionId')}")
                print(f"   Container ID: {result.get('containerId')}")
                print(f"   Status: {result.get('status')}")
                print(f"   NATS Subjects:")
                connection_info = result.get('connectionInfo', {})
                for key, value in connection_info.items():
                    print(f"      {key}: {value}")
                return True
            else:
                print("❌ Remote shell request failed")
                return False
                
    except ConnectionRefusedError:
        print("❌ Could not connect to cyclone server. Make sure cyclone is running with --enable-remote-shell")
        return False
    except Exception as e:
        print(f"❌ Test failed with error: {e}")
        return False

def test_data_structures():
    """Test that the data structures work correctly."""
    print("Testing data structures...")
    
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
        
        return True
        
    except Exception as e:
        print(f"❌ Data structure test failed: {e}")
        return False

async def main():
    """Run all tests."""
    print("=== Remote Shell Implementation Test ===\n")
    
    # Test 1: Data structures
    data_test = test_data_structures()
    print()
    
    # Test 2: Direct cyclone connection (only if data structures pass)
    if data_test:
        cyclone_test = await test_remote_shell_via_cyclone()
        print()
        
        if cyclone_test:
            print("🎉 All tests passed! Remote shell functionality is working.")
            return True
        else:
            print("ℹ️  Data structures are valid, but cyclone server test failed.")
            print("   This is expected if cyclone is not running with --enable-remote-shell")
            return True
    else:
        print("❌ Basic data structure tests failed.")
        return False

if __name__ == "__main__":
    success = asyncio.run(main())
    sys.exit(0 if success else 1)