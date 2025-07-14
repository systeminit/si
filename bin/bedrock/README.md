# Bedrock Server

Bedrock is a test infrastructure service for the System Initiative platform that provides recording, preparation, execution, and publishing capabilities for test artifacts with database state management and NATS message recording/replay.

* TLDR - Want to create a restore point for your local stack? I.e. seed some test condition in a workspace and want to restore back to it?
- Locally if you run the localhost:3020/start endpoint with a 
```json
{
  "recording_id": "<your snapshot name here>",
  "nats": [],
  "postgres": [
    "si",
    "si_layer_db"
  ],
  "metadata": {
    "messages": 0,
    "timeout": 0
  }
}
```
- Then at any point in the future you can run localhost:3020/prepare
```json
{
  "recording_id": "<your snapshot name here>",
  "metadata": {
    "messages": 0,
    "timeout": 0
  }
}
```
- If your application services are active when running the prepare, you `may` need to restart them as preparing disconnects all active postgres clients.


## Overview

The Bedrock server is built with Rust using the Axum web framework and provides HTTP endpoints for managing test environments, recording system state, and executing tests. It integrates with PostgreSQL databases, NATS message streaming, and AWS S3 for artifact storage.

## Server Configuration

- **Default Address**: `0.0.0.0:3020`
- **Framework**: Axum (Rust web framework)
- **Database**: PostgreSQL
- **Message Queue**: NATS
- **Storage**: AWS S3
- **Binary Location**: `/Users/johnwatson/working-folder/2-repos/si/bin/bedrock`
- **Library Location**: `/Users/johnwatson/working-folder/2-repos/si/lib/bedrock-server`

## Command Line Options

```bash
bedrock [OPTIONS]

Options:
  -v, --verbose                     Sets the verbosity mode (max 6)
      --no-color                    Disables ANSI coloring in log output
      --force-color                 Forces ANSI coloring
      --log-json                    Prints telemetry logging as JSON lines
      --log-file-directory <DIR>    Additionally appends logging to rolling files
      --socket-addr <ADDR>          The address and port to bind HTTP server to
      --nats-url <URL>              NATS connection URL
      --nats-creds <CREDS>          NATS credentials string
      --nats-creds-path <PATH>      NATS credentials file
      --aws-secret-access-key <KEY> AWS secret access key
      --aws-access-key-id <ID>      AWS access key ID
      --aws-session-token <TOKEN>   AWS session token
```

## HTTP Routes

### 1. System Status

**Endpoint:** `GET /`

**Purpose:** Health check endpoint

**Response:**
```json
{
  "ok": true
}
```

**Handler:** `system_status_route()` in `src/routes.rs:44`

---

### 2. List Test Profiles

**Endpoint:** `GET /profiles`

**Purpose:** Lists all available test profiles

**Response:** `TestProfileResponse` containing available test profiles

**Handler:** `profiles_route()` in `src/routes/profiles.rs:15`

**Example Response:**
```json
{
  "profiles": [
    {
      "name": "profile_name",
      "description": "Profile description"
    }
  ]
}
```

---

### 3. Execute Tests

**Endpoint:** `POST /tests`

**Purpose:** Executes a registered test profile

**Request Body:**
```json
{
  "recording_id": "string",
  "parameters": {
    // Test parameters
  },
  "execution_parameters": {
    // Execution-specific parameters
  }
}
```

**Response:** `TestResult` with execution status and details

**Success Response:**
```json
{
  "success": true,
  "message": "Test execution completed",
  "duration_ms": 1234,
  "output": "Test output details"
}
```

**Error Response:**
```json
{
  "success": false,
  "message": "Error description",
  "duration_ms": 1234,
  "output": null
}
```

**Status Codes:**
- `200 OK`: Test executed successfully
- `404 NOT_FOUND`: Test not found
- `424 FAILED_DEPENDENCY`: Service dependency issues (missing rebase batch, timeout, invalid, not found)
- `500 INTERNAL_SERVER_ERROR`: Server errors

**Handler:** `execute_tests_route()` in `src/routes/tests.rs:30`

---

### 4. Prepare Test Environment

**Endpoint:** `POST /prepare`

**Purpose:** Prepares test environment by setting up databases and clearing NATS

**Request Body:**
```json
{
  "recording_id": "string",
  "metadata": {
    "messages": 100,
    "timeout": 30
  }
}
```

**Response:** `PrepareResult` with preparation status

**Success Response:**
```json
{
  "success": true,
  "message": "Preparation complete for {recording_id}, please conduct the test and hit /tests to execute test",
  "recording_id": "string",
  "duration_ms": 1234,
  "output": null
}
```

**Error Response:**
```json
{
  "success": false,
  "message": "Error description",
  "recording_id": "string",
  "duration_ms": 1234,
  "output": null
}
```

**Status Codes:**
- `200 OK`: Preparation successful
- `404 NOT_FOUND`: Recording ID not found
- `500 INTERNAL_SERVER_ERROR`: Database or NATS preparation failed

**Handler:** `prepare_route()` in `src/routes/prepare.rs:27`

---

### 5. Start Recording

**Endpoint:** `POST /start`

**Purpose:** Starts recording artifacts (NATS streams and database dumps)

**Request Body:**
```json
{
  "recording_id": "string",  // Optional - will generate UUID if not provided
  "nats": [
    "stream1",
    "stream2"
  ],
  "postgres": [
    "database1",
    "database2"
  ],
  "metadata": {
    "messages": 100,
    "timeout": 30
  }
}
```

**Response:** `RecordResult` with recording start status

**Success Response:**
```json
{
  "success": true,
  "message": "Recording started for {recording_id}, please conduct the test and hit /stop to finalise capture",
  "recording_id": "string",
  "duration_ms": 1234,
  "output": null
}
```

**Error Response:**
```json
{
  "success": false,
  "message": "Error description",
  "recording_id": "string",
  "duration_ms": 1234,
  "output": null
}
```

**Status Codes:**
- `200 OK`: Recording started successfully
- `500 INTERNAL_SERVER_ERROR`: NATS setup or database dump failed

**Handler:** `start_recording_route()` in `src/routes/record.rs:25`

---

### 6. Stop Recording

**Endpoint:** `POST /stop`

**Purpose:** Stops recording and captures final artifacts

**Request Body:**
```json
{
  "recording_id": "string",  // Optional - will generate UUID if not provided
  "nats": [
    "stream1",
    "stream2"
  ],
  "postgres": [
    "database1",
    "database2"
  ],
  "metadata": {
    "messages": 100,
    "timeout": 30
  }
}
```

**Response:** `RecordResult` with recording stop status

**Success Response:**
```json
{
  "success": true,
  "message": "Recording stopped, please see output directory for content for recording_id {recording_id}",
  "recording_id": "string",
  "duration_ms": 1234,
  "output": null
}
```

**Error Response:**
```json
{
  "success": false,
  "message": "Error description",
  "recording_id": "string",
  "duration_ms": 1234,
  "output": null
}
```

**Status Codes:**
- `200 OK`: Recording stopped successfully
- `500 INTERNAL_SERVER_ERROR`: NATS capture or database dump failed

**Handler:** `stop_recording_route()` in `src/routes/record.rs:87`

---

### 7. Publish Artifacts

**Endpoint:** `POST /publish`

**Purpose:** Publishes captured artifacts to S3 storage

**Request Body:**
```json
{
  "recording_id": "string"
}
```

**Response:** `PublishResult` with publish status

**Success Response:**
```json
{
  "success": true,
  "message": "Artifacts published successfully",
  "duration_ms": 1234,
  "output": "S3 upload details"
}
```

**Error Response:**
```json
{
  "success": false,
  "message": "Error description",
  "duration_ms": 1234,
  "output": null
}
```

**Status Codes:**
- `200 OK`: Artifacts published successfully
- `400 BAD_REQUEST`: Invalid or missing AWS credentials
- `404 NOT_FOUND`: Artifact doesn't exist locally
- `409 CONFLICT`: Upload already exists
- `424 FAILED_DEPENDENCY`: Upload timeout
- `500 INTERNAL_SERVER_ERROR`: Other server errors

**Handler:** `publish_route()` in `src/routes/publish.rs:56`

**S3 Configuration:**
- **Bucket**: `si-artifacts-prod` within `si-shared-prod`
- **Credentials**: Configured via CLI arguments or environment variables (use DeveloperAccess for publishing)

---

## Environment Variables

- `SI_NO_COLOR`: Disable ANSI coloring
- `SI_FORCE_COLOR`: Force ANSI coloring
- `SI_LOG_JSON`: Enable JSON logging
- `SI_LOG_FILE_DIRECTORY`: Log file directory
- `AWS_SECRET_ACCESS_KEY`: AWS secret access key
- `AWS_ACCESS_KEY_ID`: AWS access key ID
- `AWS_SESSION_TOKEN`: AWS session token

## Dependencies

Key dependencies from `Cargo.toml`:
- `axum`: Web framework
- `tokio`: Async runtime
- `serde`: JSON serialization
- `aws-sdk-s3`: S3 integration
- `si-data-nats`: NATS client
- `bedrock-core`: Core types and functionality

## Error Handling

The server implements comprehensive error handling with appropriate HTTP status codes:
- `200 OK`: Successful operations
- `400 BAD_REQUEST`: Invalid requests
- `404 NOT_FOUND`: Resource not found
- `409 CONFLICT`: Resource conflicts
- `424 FAILED_DEPENDENCY`: External service issues
- `500 INTERNAL_SERVER_ERROR`: Server-side errors

All error responses follow a consistent format with success flag, message, and optional timing information.

## Workflow

1. **Start Recording** (`POST /start`): Configure NATS streams and dump initial database state
2. **Prepare Environment** (`POST /prepare`): Set up test environment with clean databases and NATS
3. **Execute Tests** (`POST /tests`): Run the actual test using recorded artifacts
4. **Stop Recording** (`POST /stop`): Capture final NATS messages and database state
5. **Publish Artifacts** (`POST /publish`): Upload captured artifacts to S3 storage

## File Structure

```
bin/bedrock/
├── BUCK
├── Cargo.toml
├── src/
│   ├── args.rs          # Command line argument parsing
│   └── main.rs          # Application entry point

lib/bedrock-server/
├── src/
│   ├── lib.rs           # Library entry point
│   ├── server.rs        # Server implementation
│   ├── routes.rs        # Route definitions
│   ├── config.rs        # Configuration management
│   ├── app_state.rs     # Application state
│   └── routes/
│       ├── profiles.rs  # Profile listing endpoint
│       ├── tests.rs     # Test execution endpoint
│       ├── prepare.rs   # Environment preparation endpoint
│       ├── record.rs    # Recording start/stop endpoints
│       └── publish.rs   # Artifact publishing endpoint
```