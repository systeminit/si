# Luminork API Testing Framework

A Deno-based testing framework for the Luminork API server, designed to validate API functionality and assist with integration testing.

## Overview

This framework provides a structured way to test the Luminork API server endpoints, including:

- System status and authentication
- Change set operations
- Component management
- Schema queries and interactions

The framework is built with Deno for modern TypeScript support and easy execution without complex build steps.

## Getting Started

### Prerequisites

- [Deno](https://deno.land/) (v1.34.0 or later)
- Access to a running Luminork API server
- Authentication token with sufficient permissions
- Workspace ID for your tests

### Configuration

Copy the `.env.example` file to `.env` in the root directory and fill in your settings:

```bash
cp .env.example .env
```

Then edit the `.env` file with your configuration:

```env
# API Endpoint URL
LUMINORK_API_URL=http://localhost:5380

# Authentication token - required
LUMINORK_AUTH_TOKEN=your_jwt_token_here

# Workspace ID for testing - required
LUMINORK_WORKSPACE_ID=your_workspace_id

# Request timeout in milliseconds
LUMINORK_TIMEOUT=30000
```

Alternatively, you can set these values as environment variables when running the tests.

## Running Tests

### Run All Tests

```bash
deno task test
```

### Run Tests in Watch Mode (for development)

```bash
deno task test:watch
```

### Run a Specific Test

```bash
deno task test tests/system-status.test.ts
```

### Using the Convenience Script

The included `run-tests.sh` script provides an easy way to run tests with custom configuration:

```bash
./scripts/run-tests.sh --url http://localhost:5380 --token your_token --workspace your_workspace_id
```

Run `./scripts/run-tests.sh --help` for more options.

## API Structure

The Luminork API follows this structure:

- `/v1/system-status` - Check system availability
- `/v1/whoami` - Verify authentication and get current user info
- `/v1/w/{workspace_id}/change-sets` - Work with change sets
- `/v1/w/{workspace_id}/schemas` - Work with schemas
- `/v1/w/{workspace_id}/change-sets/{change_set_id}/components` - Work with components

All endpoints require JWT authentication via the `Authorization: Bearer {token}` header.

## Test Structure

The test suite is organized as follows:

- `src/`: Core framework code
  - `client.ts`: Base HTTP client for the API
  - `api/`: API-specific clients for different resources
  - `test-utils.ts`: Testing utilities and helpers
- `tests/`: Test cases
  - `system-status.test.ts`: Basic connectivity tests
  - `change-sets.test.ts`: Tests for change set operations
  - `components.test.ts`: Tests for component management
  - `schemas.test.ts`: Tests for schema operations

## Using the Framework in Your Own Tests

### Basic Example

```typescript
import { assertEquals } from "https://deno.land/std@0.220.1/assert/mod.ts";
import { createTestClient, generateTestName } from "../src/test-utils.ts";

Deno.test("My Custom Test", async () => {
  // Get configured API client and test configuration
  const { api, config } = await createTestClient();

  // Create a test change set
  const response = await api.changeSets.createChangeSet(config.workspaceId, {
    name: generateTestName("my_test"),
    description: "Created by my custom test"
  });

  // Assert response is as expected
  assertEquals(response.status, 201);

  const changeSetId = response.data.id;

  // List schemas in workspace
  const schemasResponse = await api.schemas.listSchemas(config.workspaceId);

  // Create a component in the change set
  if (schemasResponse.data.items.length > 0) {
    const schemaId = schemasResponse.data.items[0].id;

    const componentResponse = await api.components.createComponent(
      config.workspaceId,
      changeSetId,
      {
        name: "Test Component",
        schema_id: schemaId
      }
    );

    assertEquals(componentResponse.status, 201);
  }

  // Clean up resources when done
  await api.changeSets.deleteChangeSet(config.workspaceId, changeSetId);
});
```

### Handling Configuration Errors

The framework includes a `ConfigError` class to help handle missing configuration:

```typescript
import { createTestClient, ConfigError } from "../src/test-utils.ts";

Deno.test("Handle Missing Config", async () => {
  try {
    const { api, config } = await createTestClient();
    // Test code here
  } catch (error) {
    if (error instanceof ConfigError) {
      console.warn(`Skipping test due to configuration error: ${error.message}`);
      return;
    }
    throw error;
  }
});
```

## Adding New Tests

1. Create a new test file in the `tests/` directory
2. Import necessary utilities from the framework
3. Use the Deno test API to create test cases
4. Ensure proper cleanup of any resources created during tests

## Best Practices

- Always clean up resources created during tests
- Use `generateTestName()` to create unique resource names
- Check for required configuration at the start of each test
- Skip tests gracefully when required configuration is missing
- Use `try/finally` blocks to ensure cleanup runs even if tests fail
- Avoid hardcoding IDs or other environment-specific values

## Debugging Issues

If you encounter issues with the API:

1. Use the `debug-api.ts` script to directly test API endpoints
2. Check the response status codes and error messages
3. Verify your authentication token is valid
4. Ensure your workspace ID is correct
5. Look for validation errors in response data

## Resources

- [Deno Documentation](https://docs.deno.com/)
- [Deno Standard Library](https://deno.land/std)
- [Luminork API Documentation](#) (Available at `/v1/docs` on your Luminork server)
