# Testing the SI MCP Server

This directory contains a testing setup for the System Initiative MCP Server using mcp-jest. The tests make **real API calls** to System Initiative using your credentials.

## Files Created

- `package.json` - Node.js package configuration for testing dependencies
- `.env.example` - Example environment configuration file
- `test/test-runner.js` - Test runner using mcp-jest library
- `test-server.sh` - Shell wrapper script for the Deno server

## Setup

1. Copy the environment file and add your credentials:
   ```bash
   cp .env.example .env
   ```

2. Edit `.env` with your System Initiative credentials:
   ```bash
   SI_API_TOKEN=your_actual_api_token_here
   SI_BASE_URL=https://api.systeminit.com
   SI_WORKSPACE_ID=your_workspace_id_here
   ```

## Running Tests

```bash
# Run tests 
npm test
# or
deno task test
```

## What Gets Tested

The tests verify:

1. **Server Connection** - Can connect to the MCP server
2. **Tool Discovery** - Server exposes the expected tools
3. **Tool Execution** - Each tool can be called successfully:
   - `validate-credentials` - Validates your API token and workspace access
   - `component-list` - Lists components in your workspace
   - `change-set-list` - Lists change sets in your workspace
   - `change-set-create` - Creates a new change set

## Environment Variables

- `SI_API_TOKEN` - Your System Initiative API token (**required**)
- `SI_BASE_URL` - API base URL (defaults to https://api.systeminit.com)  
- `SI_WORKSPACE_ID` - Workspace ID (extracted from token if not provided)

## Configuring Tests

Edit `test/test-runner.js` to add/remove tools:

```javascript
const TEST_CONFIG = {
  tools: {
    "validate-credentials": {},
    "component-list": {},
    "change-set-list": {},
    
    // Add tools with arguments:
    "change-set-create": {
      args: { changeSetName: "Test Change Set" },
    },
    "component-create": {
      args: { 
        schemaVariantId: "your-schema-variant-id",
        componentName: "Test Component"
      }
    },
  }
};
```

## Example Output

```
Running SI MCP Server Tests...

📋 Test Configuration:
   Tools to test: 4
   1. validate-credentials
   2. component-list
   3. change-set-list
   4. change-set-create (with args)
   Timeout: 15000ms

Test Results: 12/12 tests passed

✅ All tests passed!
```

## Benefits

- **Real Integration Testing**: Tests against actual System Initiative API
- **Comprehensive Coverage**: Tests connection, discovery, and tool execution
- **Fast Feedback**: Quickly verify your MCP server is working correctly
- **Easy Setup**: Simple environment variable configuration