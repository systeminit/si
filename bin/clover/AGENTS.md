# System Initiative Assistant Guide

This is a repo for working with System Initiative infrastructure through the MCP server.

## Interacting with System Initiative

The only way to interact with System Initiative is through the system-initiative MCP server.
All infrastructure operations should use the MCP tools.

## Available MCP Tools

Use MCP tools to discover schemas, create components, and manage infrastructure.

For full documentation, see: https://docs.systeminit.com

## Generating and Uploading Specs

### Generating Specs

To generate specs for a specific provider, run:

```bash
deno task run generate-specs --provider="<provider>"
```

Replace `<provider>` with the target provider (e.g., `aws`, `google cloud`, `azure`, etc.).

Generated specs are saved to the `si-specs/` directory.

### Uploading Specs

To upload generated specs from `si-specs/` to System Initiative, use:

```bash
buck2 run //bin/si:si schema upload --from-file [filepath]
```

This will upload the spec files to make them available as schemas in System Initiative. Schemas that match components that already exist will upload as Working Copies. Existing components will need to be upgraded to take on any changes.

## Testing and Debugging

### Debugging Failed Actions

When an action fails (Create, Update, Delete, Discover, Import, etc.), **always check the func run logs** to understand the actual error:

1. Use `action-list` to find the failed action and its `funcRunId`
2. Use `func-run-get` with `logs: true` and optionally `result: true` to see:
   - The actual error message from the cloud provider
   - The function execution logs showing what happened
   - The result/output of the function

Example:
```
# List actions to find the funcRunId
action-list --changeSetId <id>

# Get the func run details with logs
func-run-get --funcRunId <id> --logs true --result true
```

**Important:** MCP tool errors (like schema validation errors) may hide the actual cloud provider error. Always dig into the func run logs to find the root cause.

### Tests to Run

Look in each pipeline for a smoke.md file that describes which tests to run for that provider.
