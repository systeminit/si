# si-tmpl

A command-line tool for running System Initiative templates.

## Prerequisites

- [Deno](https://deno.land/) runtime installed

## Installation

No installation required - run directly using Deno.

## Usage

### Running a Template

Run a template file with the `run` command:

```bash
SI_API_TOKEN=<your-token> deno run --allow-net --allow-env --allow-read --allow-write main.ts run <template-file> --key <invocation-key>
```

Or use the deno task:

```bash
SI_API_TOKEN=<your-token> deno task dev run <template-file> --key <invocation-key>
```

**Arguments:**

- `<template-file>` - Path to your template TypeScript file
- `--key <invocation-key>` - Required invocation key for idempotency control

**Environment Variables:**

- `SI_API_TOKEN` - **Required** System Initiative API authentication token (JWT)
- `SI_BASE_URL` - Optional API base URL (defaults to
  `https://api.systeminit.com`)

**Example:**

```bash
SI_API_TOKEN=eyJhbGc... deno run --allow-net --allow-env --allow-read --allow-write main.ts run ./tmpl/test.ts --key my-unique-key
```


### Generating a baseline cache

To generate a baseline cache of your workspace, create a template file with search strings:

```typescript
// tmpl/cache.ts
import { TemplateContext } from "../src/template.ts";

export default async function (ctx: TemplateContext) {
  ctx.search(["schema:*"]);
}
```

Then cache your workspace:

```bash
deno main.ts run ./tmpl/cache.ts --key cache-gen --cache-baseline ./cache/baseline.yaml --cache-baseline-only
```

Use the cached baseline in subsequent template runs:

```bash
deno main.ts run <your-template-file> --key <invocation-key> --baseline ./cache/baseline.yaml
```

The cache file format (`.json` or `.yaml`) is determined by the file extension.

### Options

**Global Options:**

- `-v, --verbose [level]` - Enable verbose logging (0-4, default: 2)
  - 0: errors only
  - 1: + warnings
  - 2: + info (default)
  - 3: + debug
  - 4: + trace
- `--no-color` - Disable colored output

**Example with verbose logging:**

```bash
deno task dev run ./tmpl/test.ts --key my-key --verbose 3
```

## Writing Templates

Templates are TypeScript files that export a default function receiving a
`TemplateContext`:

```typescript
import { TemplateContext } from "../src/template.ts";
import { ComponentsApi } from "@systeminit/api-client";
import { z } from "zod";

export default async function (ctx: TemplateContext) {
  // Set template name
  ctx.name("my-template");

  // Set change set name
  ctx.changeSet("my-changeset");

  // Set search strings
  ctx.search(["schema:*", "component:aws"]);

  // Set name pattern for transforming component names (s/prod-(.+)/staging-$1/g)
  ctx.namePattern({
    pattern: /prod-(.+)/g,
    replacement: "staging-$1",
  });

  // Set input schema with defaults
  ctx.inputs(z.object({
    environment: z.string().default("production"),
    replicas: z.number().default(3),
  }));

  // Set transform function for the working set
  ctx.transform((workingSet) => {
    // Filter to only production components
    return workingSet.filter((c) => c.name.startsWith("prod-"));
  });

  // Access the invocation key
  const key = ctx.invocationKey();

  // Access System Initiative API client
  const workspaceId = ctx.workspaceId();
  const apiConfig = ctx.apiConfig();

  if (workspaceId && apiConfig) {
    // Use the API client to interact with System Initiative
    const componentsApi = new ComponentsApi(apiConfig);
    // Example: list components in the workspace
    // const response = await componentsApi.listComponents({
    //   workspaceId,
    //   changeSetId: "your-changeset-id"
    // });
  }

  // Use the logger
  ctx.logger.info("Template executing");
}
```

### TemplateContext API

**Configuration Methods:**

- `name()` / `name(newName)` - Get or set the template name
- `changeSet()` / `changeSet(newName)` - Get or set the change set name
- `search()` / `search(searchArray)` - Get or set the search strings for finding
  components
- `namePattern()` / `namePattern(pattern)` - Get or set the name pattern for
  transforming component names
- `inputs()` / `inputs(schema)` - Get or set the input schema using Zod for
  validation
- `inputData()` - Get the validated input data (after inputs have been loaded)
- `transform()` / `transform(fn)` - Get or set the transform function for the
  working set

**Data Access Methods:**

- `baseline()` - Get the baseline components (original state from search)
- `workingSet()` - Get the working set (modified components to be converged)
- `schemaCache()` - Get the schema cache Map

**API Access Methods:**

- `apiConfig()` - Get the System Initiative API client configuration (read-only)
- `workspaceId()` - Get the workspace ID from the API token (read-only)
- `userId()` - Get the user ID from the API token (read-only)
- `invocationKey()` - Get the invocation key (read-only)
- `getHeadChangeSetId()` - Get the HEAD change set ID for the workspace

**Schema Helper Methods:**

- `getSchemaName(workspaceId, changeSetId, schemaId)` - Get human-readable
  schema name from ID (e.g., "AWS EC2 Instance")
- `getSchemaIdByName(workspaceId, changeSetId, schemaName)` - Get schema ID from
  name

**Component Manipulation Methods:**

- `setAttribute(component, path, value)` - Set an attribute value on a component
- `deleteAttribute(component, path)` - Delete an attribute from a component
- `setSiblingAttribute(component, sourcePath, targetPath, value)` - Set an
  attribute relative to another attribute's parent
- `setSubscription(component, path, subscription)` - Set a subscription to
  receive values from another component
- `copyComponent(source, newName)` - Create a deep copy of a component with a
  new name
- `newComponent(name, schemaId)` - Create a new component from scratch

**Logging:**

- `logger` - Access the logger instance for logging messages

## Development

### Run Tests

```bash
deno test --allow-env
```

### Build

```bash
deno task build
```

This creates a standalone executable.

### Lint

```bash
deno task lint
```

## Shell Completions

Generate shell completions for bash, zsh, or fish:

```bash
deno run --allow-net --allow-env --allow-read --allow-write main.ts completion bash > completions.bash
source completions.bash
```
