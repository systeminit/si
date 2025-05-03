---
outline:
  level: [2, 3, 4]
---

# System Initiative API Documentation

## Overview

The External API provides programmatic access to System Initiative's platform
resources. This API follows RESTful principles and uses JSON for request and
response payloads.

**API Version:** 1.0.0 **Base URL:** `/`

## Authentication

Most endpoints require authentication. Use the `/whoami` endpoint to verify your
authentication status and retrieve your user information.

## Common Response Codes

| Code | Description                             |
| ---- | --------------------------------------- |
| 200  | Request succeeded                       |
| 401  | Unauthorized - Invalid or missing token |
| 403  | Forbidden - Insufficient permissions    |
| 404  | Resource not found                      |
| 422  | Validation error - Invalid request data |
| 500  | Internal server error                   |
| 503  | Service in maintenance mode             |

## Error Responses

Error responses follow a standard format:

```json
{
  "message": "Error description",
  "statusCode": 500,
  "code": 1234 // Optional error code
}
```

---

## Endpoints

### System Status

#### Get System Status

Retrieves the current system status information.

```
GET /
```

**Responses:**

| Status | Description                 |
| ------ | --------------------------- |
| 200    | System status information   |
| 503    | Service in maintenance mode |

**Example Response (200):**

```json
{
  "What is this?": "I am luminork, the new System Initiative External API server",
  "API Documentation": "Available at /swagger-ui"
}
```

### User Identity

#### Get Current User Information

Retrieves information about the currently authenticated user.

```
GET /whoami
```

**Responses:**

| Status | Description                             |
| ------ | --------------------------------------- |
| 200    | Successfully retrieved user information |
| 401    | Unauthorized - Invalid or expired token |
| 403    | Forbidden - Insufficient permissions    |

**Example Response (200):**

```json
{
  "userId": "01H9ZQCBJ3E7HBTRN3J58JQX8K",
  "userEmail": "user@example.com",
  "workspaceId": "01H9ZQD35JPMBGHH69BT0Q79VY",
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

### Change Sets

#### List Change Sets

Retrieves a list of change sets for a workspace.

```
GET /v1/w/{workspace_id}/change-sets
```

**Parameters:**

| Name         | In   | Type   | Required | Description          |
| ------------ | ---- | ------ | -------- | -------------------- |
| workspace_id | path | string | Yes      | Workspace identifier |

**Responses:**

| Status | Description                     |
| ------ | ------------------------------- |
| 200    | Change sets listed successfully |
| 500    | Internal server error           |

**Example Response (200):**

```json
{
  "changeSets": [
    {
      "id": "01H9ZQD35JPMBGHH69BT0Q79VY",
      "name": "Add new feature",
      "status": "Draft"
    }
  ]
}
```

#### Create Change Set

Creates a new change set in a workspace.

```
POST /v1/w/{workspace_id}/change-sets
```

**Parameters:**

| Name         | In   | Type   | Required | Description          |
| ------------ | ---- | ------ | -------- | -------------------- |
| workspace_id | path | string | Yes      | Workspace identifier |

**Request Body:**

```json
{
  "changeSetName": "My new feature"
}
```

**Responses:**

| Status | Description                             |
| ------ | --------------------------------------- |
| 200    | Change set created successfully         |
| 422    | Validation error - Invalid request data |
| 500    | Internal server error                   |

#### Get Change Set

Retrieves a specific change set.

```
GET /v1/w/{workspace_id}/change-sets/{change_set_id}
```

**Parameters:**

| Name          | In   | Type   | Required | Description           |
| ------------- | ---- | ------ | -------- | --------------------- |
| workspace_id  | path | string | Yes      | Workspace identifier  |
| change_set_id | path | string | Yes      | Change set identifier |

**Responses:**

| Status | Description                       |
| ------ | --------------------------------- |
| 200    | Change set retrieved successfully |
| 500    | Internal server error             |

#### Abandon Change Set

Deletes/abandons a change set.

```
DELETE /v1/w/{workspace_id}/change-sets/{change_set_id}
```

**Parameters:**

| Name          | In   | Type   | Required | Description           |
| ------------- | ---- | ------ | -------- | --------------------- |
| workspace_id  | path | string | Yes      | Workspace identifier  |
| change_set_id | path | string | Yes      | Change set identifier |

**Responses:**

| Status | Description                     |
| ------ | ------------------------------- |
| 200    | Change set deleted successfully |
| 500    | Internal server error           |

**Example Response (200):**

```json
{
  "success": true
}
```

#### Force Apply Change Set

Forces the application of a change set.

```
POST /v1/w/{workspace_id}/change-sets/{change_set_id}/force_apply
```

**Parameters:**

| Name          | In   | Type   | Required | Description           |
| ------------- | ---- | ------ | -------- | --------------------- |
| workspace_id  | path | string | Yes      | Workspace identifier  |
| change_set_id | path | string | Yes      | Change set identifier |

**Responses:**

| Status | Description                           |
| ------ | ------------------------------------- |
| 200    | Change set force applied successfully |
| 500    | Internal server error                 |

#### Request Change Set Approval

Requests approval for a change set.

```
POST /v1/w/{workspace_id}/change-sets/{change_set_id}/request_approval
```

**Parameters:**

| Name          | In   | Type   | Required | Description           |
| ------------- | ---- | ------ | -------- | --------------------- |
| workspace_id  | path | string | Yes      | Workspace identifier  |
| change_set_id | path | string | Yes      | Change set identifier |

**Responses:**

| Status | Description                                |
| ------ | ------------------------------------------ |
| 200    | Change set approval requested successfully |
| 500    | Internal server error                      |

#### Get Change Set Merge Status

Retrieves the merge status of a change set.

```
GET /v1/w/{workspace_id}/change-sets/{change_set_id}/merge_status
```

**Parameters:**

| Name          | In   | Type   | Required | Description           |
| ------------- | ---- | ------ | -------- | --------------------- |
| workspace_id  | path | string | Yes      | Workspace identifier  |
| change_set_id | path | string | Yes      | Change set identifier |

**Responses:**

| Status | Description                                    |
| ------ | ---------------------------------------------- |
| 200    | Change set merge status retrieved successfully |
| 500    | Internal server error                          |

### Components

#### List Components

Retrieves a list of components for a change set.

```
GET /v1/w/{workspace_id}/change-sets/{change_set_id}/components
```

**Parameters:**

| Name          | In   | Type   | Required | Description           |
| ------------- | ---- | ------ | -------- | --------------------- |
| workspace_id  | path | string | Yes      | Workspace identifier  |
| change_set_id | path | string | Yes      | Change set identifier |

**Responses:**

| Status | Description                             |
| ------ | --------------------------------------- |
| 200    | Components retrieved successfully       |
| 401    | Unauthorized - Invalid or missing token |
| 500    | Internal server error                   |

#### Create Component

Creates a new component in a change set.

```
POST /v1/w/{workspace_id}/change-sets/{change_set_id}/components
```

**Parameters:**

| Name          | In   | Type   | Required | Description           |
| ------------- | ---- | ------ | -------- | --------------------- |
| workspace_id  | path | string | Yes      | Workspace identifier  |
| change_set_id | path | string | Yes      | Change set identifier |

**Request Body:**

```json
{
  "name": "MyComponentName",
  "schemaName": "AWS::EC2::Instance",
  "domain": {
    "propId1": "value1",
    "path/to/prop": "value2"
  },
  "connections": [
    {
      "from": {
        "component": "OtherComponentName",
        "socketName": "SocketName"
      },
      "to": "ThisComponentInputSocketName"
    }
  ],
  "secrets": {
    "secretDefinitionName": "secretName"
  },
  "viewName": "MyView"
}
```

**Responses:**

| Status | Description                             |
| ------ | --------------------------------------- |
| 200    | Component created successfully          |
| 401    | Unauthorized - Invalid or missing token |
| 404    | Component not found                     |
| 412    | Precondition Failed - View not found    |
| 422    | Validation error - Invalid request data |
| 500    | Internal server error                   |

#### Find Component

Finds a component by name or ID.

```
GET /v1/w/{workspace_id}/change-sets/{change_set_id}/components/find
```

**Parameters:**

| Name          | In    | Type   | Required | Description           |
| ------------- | ----- | ------ | -------- | --------------------- |
| workspace_id  | path  | string | Yes      | Workspace identifier  |
| change_set_id | path  | string | Yes      | Change set identifier |
| component     | query | string | No       | Component name        |
| componentId   | query | string | No       | Component ID          |

**Responses:**

| Status | Description                             |
| ------ | --------------------------------------- |
| 200    | Component retrieved successfully        |
| 401    | Unauthorized - Invalid or missing token |
| 404    | Component not found                     |
| 500    | Internal server error                   |

#### Get Component

Retrieves a specific component.

```
GET /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}
```

**Parameters:**

| Name          | In   | Type   | Required | Description           |
| ------------- | ---- | ------ | -------- | --------------------- |
| workspace_id  | path | string | Yes      | Workspace identifier  |
| change_set_id | path | string | Yes      | Change set identifier |
| component_id  | path | string | Yes      | Component identifier  |

**Responses:**

| Status | Description                             |
| ------ | --------------------------------------- |
| 200    | Component retrieved successfully        |
| 401    | Unauthorized - Invalid or missing token |
| 404    | Component not found                     |
| 500    | Internal server error                   |

#### Update Component

Updates a specific component.

```
PUT /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}
```

**Parameters:**

| Name          | In   | Type   | Required | Description           |
| ------------- | ---- | ------ | -------- | --------------------- |
| workspace_id  | path | string | Yes      | Workspace identifier  |
| change_set_id | path | string | Yes      | Change set identifier |
| component_id  | path | string | Yes      | Component identifier  |

**Request Body:**

```json
{
  "name": "MyUpdatedComponentName",
  "domain": {
    "propId1": "value1",
    "path/to/prop": "value2"
  },
  "connectionChanges": {
    "add": [
      {
        "from": {
          "component": "OtherComponentName",
          "socketName": "output"
        },
        "to": "ThisComponentInputSocketName"
      }
    ],
    "remove": [
      {
        "from": {
          "componentId": "01H9ZQD35JPMBGHH69BT0Q79VY",
          "socketName": "output"
        },
        "to": "ThisComponentInputSocketName"
      }
    ]
  },
  "secrets": {
    "secretDefinitionName": "secretName"
  },
  "unset": ["propId1", "path/to/prop"]
}
```

**Responses:**

| Status | Description                                    |
| ------ | ---------------------------------------------- |
| 200    | Component updated successfully                 |
| 404    | Component not found                            |
| 412    | Precondition failed - Duplicate component name |
| 422    | Validation error - Invalid request data        |
| 500    | Internal server error                          |

#### Delete Component

Marks a component for deletion.

```
DELETE /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}
```

**Parameters:**

| Name          | In   | Type   | Required | Description           |
| ------------- | ---- | ------ | -------- | --------------------- |
| workspace_id  | path | string | Yes      | Workspace identifier  |
| change_set_id | path | string | Yes      | Change set identifier |
| component_id  | path | string | Yes      | Component identifier  |

**Responses:**

| Status | Description                    |
| ------ | ------------------------------ |
| 200    | Component deleted successfully |
| 404    | Component not found            |
| 500    | Internal server error          |

**Example Response (200):**

```json
{
  "status": "MarkedForDeletion"
}
```

#### Add Action to Component

Adds an action to a component.

```
POST /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}/action
```

**Parameters:**

| Name          | In   | Type   | Required | Description           |
| ------------- | ---- | ------ | -------- | --------------------- |
| workspace_id  | path | string | Yes      | Workspace identifier  |
| change_set_id | path | string | Yes      | Change set identifier |
| component_id  | path | string | Yes      | Component identifier  |

**Request Body:**

```json
{
  "action": {
    "function": "Create Asset"
  }
}
```

**Responses:**

| Status | Description                                                     |
| ------ | --------------------------------------------------------------- |
| 200    | Action successfully queued                                      |
| 401    | Unauthorized - Invalid or missing token                         |
| 404    | Component or function not found                                 |
| 409    | Action already enqueued                                         |
| 412    | Precondition Failed - View not found or duplicate function name |
| 422    | Validation error - Invalid request data                         |
| 500    | Internal server error                                           |

#### Execute Management Function on Component

Executes a management function on a component.

```
POST /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}/execute-management-function
```

**Parameters:**

| Name          | In   | Type   | Required | Description           |
| ------------- | ---- | ------ | -------- | --------------------- |
| workspace_id  | path | string | Yes      | Workspace identifier  |
| change_set_id | path | string | Yes      | Change set identifier |
| component_id  | path | string | Yes      | Component identifier  |

**Request Body:**

```json
{
  "viewName": "MyViewName",
  "managementFunction": {
    "function": "CreateVpc"
  }
}
```

**Responses:**

| Status | Description                                                     |
| ------ | --------------------------------------------------------------- |
| 200    | Function successfully dispatched                                |
| 401    | Unauthorized - Invalid or missing token                         |
| 404    | Component or function not found                                 |
| 412    | Precondition Failed - View not found or duplicate function name |
| 422    | Validation error - Invalid request data                         |
| 500    | Internal server error                                           |

**Example Response (200):**

```json
{
  "funcRunId": "01H9ZQD35JPMBGHH69BT0Q79VY"
}
```

### Actions

#### Get Actions

Retrieves a list of actions for a change set.

```
GET /v1/w/{workspace_id}/change-sets/{change_set_id}/actions/
```

**Parameters:**

| Name          | In   | Type   | Required | Description           |
| ------------- | ---- | ------ | -------- | --------------------- |
| workspace_id  | path | string | Yes      | Workspace identifier  |
| change_set_id | path | string | Yes      | Change set identifier |

**Responses:**

| Status | Description                             |
| ------ | --------------------------------------- |
| 200    | Actions retrieved successfully          |
| 401    | Unauthorized - Invalid or missing token |
| 500    | Internal server error                   |

#### Cancel Action

Cancels a specific action.

```
POST /v1/w/{workspace_id}/change-sets/{change_set_id}/actions/{action_id}/cancel
```

**Parameters:**

| Name          | In   | Type   | Required | Description           |
| ------------- | ---- | ------ | -------- | --------------------- |
| workspace_id  | path | string | Yes      | Workspace identifier  |
| change_set_id | path | string | Yes      | Change set identifier |
| action_id     | path | string | Yes      | Action identifier     |

**Responses:**

| Status | Description                             |
| ------ | --------------------------------------- |
| 200    | Action cancelled successfully           |
| 401    | Unauthorized - Invalid or missing token |
| 404    | Action not found                        |
| 500    | Internal server error                   |

**Example Response (200):**

```json
{
  "success": true
}
```

#### Put Action on Hold

Places an action on hold.

```
POST /v1/w/{workspace_id}/change-sets/{change_set_id}/actions/{action_id}/put_on_hold
```

**Parameters:**

| Name          | In   | Type   | Required | Description           |
| ------------- | ---- | ------ | -------- | --------------------- |
| workspace_id  | path | string | Yes      | Workspace identifier  |
| change_set_id | path | string | Yes      | Change set identifier |
| action_id     | path | string | Yes      | Action identifier     |

**Responses:**

| Status | Description                             |
| ------ | --------------------------------------- |
| 200    | Action successfully put on hold         |
| 401    | Unauthorized - Invalid or missing token |
| 404    | Action not found                        |
| 500    | Internal server error                   |

**Example Response (200):**

```json
{
  "success": true
}
```

#### Retry Action

Retries a specific action.

```
POST /v1/w/{workspace_id}/change-sets/{change_set_id}/actions/{action_id}/retry
```

**Parameters:**

| Name          | In   | Type   | Required | Description           |
| ------------- | ---- | ------ | -------- | --------------------- |
| workspace_id  | path | string | Yes      | Workspace identifier  |
| change_set_id | path | string | Yes      | Change set identifier |
| action_id     | path | string | Yes      | Action identifier     |

**Responses:**

| Status | Description                             |
| ------ | --------------------------------------- |
| 200    | Action successfully requeued            |
| 401    | Unauthorized - Invalid or missing token |
| 404    | Action not found                        |
| 500    | Internal server error                   |

**Example Response (200):**

```json
{
  "success": true
}
```

### Functions

#### Get Function

Retrieves details about a specific function.

```
GET /v1/w/{workspace_id}/change-sets/{change_set_id}/funcs/{func_id}
```

**Parameters:**

| Name          | In   | Type   | Required | Description           |
| ------------- | ---- | ------ | -------- | --------------------- |
| workspace_id  | path | string | Yes      | Workspace identifier  |
| change_set_id | path | string | Yes      | Change set identifier |
| func_id       | path | string | Yes      | Function identifier   |

**Responses:**

| Status | Description                             |
| ------ | --------------------------------------- |
| 200    | Function retrieved successfully         |
| 401    | Unauthorized - Invalid or missing token |
| 404    | Function not found                      |
| 500    | Internal server error                   |

#### Get Function Run

Retrieves details about a specific function run.

```
GET /v1/w/{workspace_id}/change-sets/{change_set_id}/funcs/runs/{func_run_id}
```

**Parameters:**

| Name          | In   | Type   | Required | Description             |
| ------------- | ---- | ------ | -------- | ----------------------- |
| workspace_id  | path | string | Yes      | Workspace identifier    |
| change_set_id | path | string | Yes      | Change set identifier   |
| func_run_id   | path | string | Yes      | Function run identifier |

**Responses:**

| Status | Description                             |
| ------ | --------------------------------------- |
| 200    | Function run retrieved successfully     |
| 401    | Unauthorized - Invalid or missing token |
| 404    | Function run not found                  |
| 500    | Internal server error                   |

### Schemas

#### List Schemas

Retrieves a list of schemas for a change set.

```
GET /v1/w/{workspace_id}/change-sets/{change_set_id}/schema
```

**Parameters:**

| Name          | In   | Type   | Required | Description           |
| ------------- | ---- | ------ | -------- | --------------------- |
| workspace_id  | path | string | Yes      | Workspace identifier  |
| change_set_id | path | string | Yes      | Change set identifier |

**Responses:**

| Status | Description                             |
| ------ | --------------------------------------- |
| 200    | Schemas listed successfully             |
| 401    | Unauthorized - Invalid or missing token |
| 500    | Internal server error                   |

**Example Response (200):**

```json
{
  "schemas": [
    {
      "schemaId": "01H9ZQD35JPMBGHH69BT0Q79VY",
      "schemaName": "AWS::EC2::Instance",
      "category": "AWS::EC2",
      "installed": "true"
    }
  ]
}
```

#### Find Schema

Finds a schema by name or ID.

```
GET /v1/w/{workspace_id}/change-sets/{change_set_id}/schema/find
```

**Parameters:**

| Name          | In    | Type   | Required | Description           |
| ------------- | ----- | ------ | -------- | --------------------- |
| workspace_id  | path  | string | Yes      | Workspace identifier  |
| change_set_id | path  | string | Yes      | Change set identifier |
| schema        | query | string | No       | Schema name           |
| schemaId      | query | string | No       | Schema ID             |

**Responses:**

| Status | Description                             |
| ------ | --------------------------------------- |
| 200    | Schema retrieved successfully           |
| 401    | Unauthorized - Invalid or missing token |
| 404    | Schema not found                        |
| 500    | Internal server error                   |

#### Get Schema

Retrieves a specific schema.

```
GET /v1/w/{workspace_id}/change-sets/{change_set_id}/schema/{schema_id}
```

**Parameters:**

| Name          | In   | Type   | Required | Description           |
| ------------- | ---- | ------ | -------- | --------------------- |
| workspace_id  | path | string | Yes      | Workspace identifier  |
| change_set_id | path | string | Yes      | Change set identifier |
| schema_id     | path | string | Yes      | Schema identifier     |

**Responses:**

| Status | Description                             |
| ------ | --------------------------------------- |
| 200    | Schema retrieved successfully           |
| 401    | Unauthorized - Invalid or missing token |
| 404    | Schema not found                        |
| 500    | Internal server error                   |

#### Get Default Schema Variant

Retrieves the default variant of a schema.

```
GET /v1/w/{workspace_id}/change-sets/{change_set_id}/schema/{schema_id}/variant/default
```

**Parameters:**

| Name          | In   | Type   | Required | Description           |
| ------------- | ---- | ------ | -------- | --------------------- |
| workspace_id  | path | string | Yes      | Workspace identifier  |
| change_set_id | path | string | Yes      | Change set identifier |
| schema_id     | path | string | Yes      | Schema identifier     |

**Responses:**

| Status | Description                             |
| ------ | --------------------------------------- |
| 200    | Schema variant retrieved successfully   |
| 401    | Unauthorized - Invalid or missing token |
| 404    | Schema variant not found                |
| 500    | Internal server error                   |

#### Get Schema Variant

Retrieves a specific variant of a schema.

```
GET /v1/w/{workspace_id}/change-sets/{change_set_id}/schema/{schema_id}/variant/{schema_variant_id}
```

**Parameters:**

| Name              | In   | Type   | Required | Description               |
| ----------------- | ---- | ------ | -------- | ------------------------- |
| workspace_id      | path | string | Yes      | Workspace identifier      |
| change_set_id     | path | string | Yes      | Change set identifier     |
| schema_id         | path | string | Yes      | Schema identifier         |
| schema_variant_id | path | string | Yes      | Schema variant identifier |

**Responses:**

| Status | Description                             |
| ------ | --------------------------------------- |
| 200    | Schema variant retrieved successfully   |
| 401    | Unauthorized - Invalid or missing token |
| 404    | Schema variant not found                |
| 412    | Schema variant not found for schema     |
| 500    | Internal server error                   |

### Secrets

#### List Secrets

Retrieves a list of secrets for a change set.

```
GET /v1/w/{workspace_id}/change-sets/{change_set_id}/secrets
```

**Parameters:**

| Name          | In   | Type   | Required | Description           |
| ------------- | ---- | ------ | -------- | --------------------- |
| workspace_id  | path | string | Yes      | Workspace identifier  |
| change_set_id | path | string | Yes      | Change set identifier |

**Responses:**

| Status | Description                             |
| ------ | --------------------------------------- |
| 200    | Secrets retrieved successfully          |
| 401    | Unauthorized - Invalid or missing token |
| 500    | Internal server error                   |

#### Create Secret

Creates a new secret in a change set.

```
POST /v1/w/{workspace_id}/change-sets/{change_set_id}/secrets
```

**Parameters:**

| Name          | In   | Type   | Required | Description           |
| ------------- | ---- | ------ | -------- | --------------------- |
| workspace_id  | path | string | Yes      | Workspace identifier  |
| change_set_id | path | string | Yes      | Change set identifier |

**Request Body:**

```json
{
  "name": "MySecret",
  "definitionName": "SecretType",
  "description": "Description of the secret",
  "rawData": {
    // Secret data
  }
}
```

**Responses:**

| Status | Description                             |
| ------ | --------------------------------------- |
| 200    | Secret created successfully             |
| 401    | Unauthorized - Invalid or missing token |
| 422    | Validation error - Invalid request data |
| 500    | Internal server error                   |

#### Update Secret

Updates a specific secret.

```
PUT /v1/w/{workspace_id}/change-sets/{change_set_id}/secrets/{secret_id}
```

**Parameters:**

| Name          | In   | Type   | Required | Description           |
| ------------- | ---- | ------ | -------- | --------------------- |
| workspace_id  | path | string | Yes      | Workspace identifier  |
| change_set_id | path | string | Yes      | Change set identifier |
| secret_id     | path | string | Yes      | Secret identifier     |

**Request Body:**

```json
{
  "id": "01H9ZQD35JPMBGHH69BT0Q79VY",
  "name": "UpdatedSecretName",
  "description": "Updated description",
  "rawData": {
    // Updated secret data
  }
}
```

**Responses:**

| Status | Description                 |
| ------ | --------------------------- |
| 200    | Secret updated successfully |
| 404    | Secret not found            |
| 500    | Internal server error       |

#### Delete Secret

Deletes a specific secret.

```
DELETE /v1/w/{workspace_id}/change-sets/{change_set_id}/secrets/{secret_id}
```

**Parameters:**

| Name          | In   | Type   | Required | Description           |
| ------------- | ---- | ------ | -------- | --------------------- |
| workspace_id  | path | string | Yes      | Workspace identifier  |
| change_set_id | path | string | Yes      | Change set identifier |
| secret_id     | path | string | Yes      | Secret identifier     |

**Responses:**

| Status | Description                 |
| ------ | --------------------------- |
| 200    | Secret deleted successfully |
| 404    | Secret not found            |
| 500    | Internal server error       |

**Example Response (200):**

```json
{
  "success": true
}
```

## Data Models

This section describes the key data structures used by the API.

### ActionReference

A reference to a management function by either name or ID.

```json
// By function name
{
  "function": "Create Asset"
}

// By action prototype ID
{
  "actionPrototypeId": "01H9ZQD35JPMBGHH69BT0Q79VY"
}
```

### ManagementFunctionReference

A reference to a management function by either name or ID.

```json
// By function name
{
  "function": "CreateVpc"
}

// By management prototype ID
{
  "managementPrototypeId": "01H9ZQD35JPMBGHH69BT0Q79VY"
}
```

### ComponentReference

A reference to a component by either name or ID.

```json
// By component name
{
  "component": "MyComponent"
}

// By component ID
{
  "componentId": "01H9ZQD35JPMBGHH69BT0Q79VY"
}
```

### ConnectionPoint

A reference to a socket on a component.

```json
{
  "componentId": "01H9ZQD35JPMBGHH69BT0Q79VY", // or "component": "ComponentName"
  "socketName": "OutputSocketName"
}
```

### Connection

A connection between two components.

```json
// From another component to this component
{
  "from": {
    "componentId": "01H9ZQD35JPMBGHH69BT0Q79VY",
    "socketName": "output"
  },
  "to": "ThisComponentInputSocketName"
}

// From this component to another component
{
  "from": "ThisComponentOutputSocketName",
  "to": {
    "componentId": "01H9ZQD35JPMBGHH69BT0Q79VY",
    "socketName": "InputSocketName"
  }
}
```

### ConnectionDetails

Details for updating connections.

```json
{
  "add": [
    {
      "from": {
        "component": "OtherComponentName",
        "socketName": "output"
      },
      "to": "ThisComponentInputSocketName"
    }
  ],
  "remove": [
    {
      "from": {
        "componentId": "01H9ZQD35JPMBGHH69BT0Q79VY",
        "socketName": "output"
      },
      "to": "ThisComponentInputSocketName"
    }
  ]
}
```
