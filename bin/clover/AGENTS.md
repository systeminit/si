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
