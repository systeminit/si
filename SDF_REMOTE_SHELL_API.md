# SDF Remote Shell API

## Overview

A new API endpoint has been added to SDF that allows creating remote shell sessions through the veritech infrastructure. The endpoint handles the creation of remote shell sessions and returns session details including NATS communication subjects.

## API Endpoint

### POST `/api/v2/workspaces/{workspace_id}/change-sets/{change_set_id}/remote-shell/create`

Creates a new remote shell session.

#### Request Body

```json
{
  "image": "ubuntu:20.04",              // Optional: Docker image to use
  "workingDir": "/workspace",           // Optional: Working directory in container
  "envVars": {                          // Optional: Environment variables
    "USER": "developer",
    "PATH": "/usr/local/bin:/usr/bin:/bin"
  }
}
```

#### Response

```json
{
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
    "message": "Remote shell session placeholder - not yet fully implemented"
  }
}
```

#### Error Response

```json
{
  "error": {
    "message": "Error description",
    "statusCode": 500
  }
}
```

## NATS Communication Subjects

Once a remote shell session is created, you can interact with it using the provided NATS subjects:

- **`stdinSubject`**: Send input commands to the shell
- **`stdoutSubject`**: Receive shell output
- **`stderrSubject`**: Receive shell error output  
- **`controlSubject`**: Send control commands (resize, terminate, etc.)

## Usage Example

### 1. Create Remote Shell Session

```bash
curl -X POST \
  "https://api.systeminit.com/api/v2/workspaces/my-workspace/change-sets/my-changeset/remote-shell/create" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{
    "image": "ubuntu:20.04",
    "workingDir": "/workspace",
    "envVars": {
      "USER": "developer"
    }
  }'
```

### 2. Use Session Details

The response contains NATS subjects that can be used to interact with the shell session:

```javascript
// Example using NATS client
const nats = await connect({ servers: 'nats://localhost:4222' });

// Send command to shell
await nats.publish('remote_shell.EXECUTION_ID.stdin', 'ls -la\n');

// Subscribe to output
const sub = nats.subscribe('remote_shell.EXECUTION_ID.stdout');
for await (const m of sub) {
  console.log('Shell output:', new TextDecoder().decode(m.data));
}
```

## Implementation Notes

### Current Status
- ✅ API endpoint implementation complete
- ✅ Integration with veritech-client
- ✅ Request/response serialization
- ✅ Error handling
- 🚧 Placeholder response from cyclone (actual container spawning not yet implemented)

### Architecture Flow

1. **SDF API** receives POST request to create remote shell
2. **Request Processing** validates input and creates `RemoteShellRequest`
3. **Veritech Client** sends request to veritech via NATS JetStream
4. **Veritech Server** routes request to appropriate cyclone worker
5. **Cyclone Worker** handles remote shell creation (currently placeholder)
6. **Response** returns session details and NATS communication subjects

### Security Considerations

- Endpoint requires proper workspace and changeset authorization
- All requests are tracked via PostHog analytics
- Session lifecycle should be managed properly to prevent resource leaks

## Testing

### Prerequisites
- Running SDF server with remote shell support
- Running veritech with remote shell enabled cyclone workers
- NATS server for communication
- Valid authentication tokens

### Manual Testing

1. **Start Services**:
   ```bash
   # Start cyclone with remote shell support
   cargo run --bin cyclone -- --enable-remote-shell --enable-watch
   
   # Start veritech
   cargo run --bin veritech
   
   # Start SDF
   cargo run --bin sdf
   ```

2. **Test API Call**:
   ```bash
   curl -X POST \
     "http://localhost:5156/api/v2/workspaces/test-workspace/change-sets/test-changeset/remote-shell/create" \
     -H "Content-Type: application/json" \
     -d '{"image": "ubuntu:20.04"}'
   ```

3. **Verify Response**: Check that response contains valid session details and NATS subjects

## Next Steps

To make the remote shell functionality fully operational:

1. **Container Management**: Implement actual Docker container spawning in cyclone remote shell handler
2. **I/O Bridging**: Connect container stdin/stdout to NATS subjects  
3. **Session Lifecycle**: Add session timeout, cleanup, and proper resource management
4. **Authentication**: Ensure remote shell sessions are properly authenticated
5. **Resource Limits**: Apply container resource constraints for security

## Integration

This API integrates seamlessly with existing SDF infrastructure:
- Uses existing authentication and authorization
- Follows SDF API patterns and error handling
- Leverages existing veritech task execution system
- Maintains audit trail and analytics tracking