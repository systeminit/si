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
deno run --allow-net --allow-env --allow-read --allow-write main.ts run <template-file> --key <invocation-key>
```

Or use the deno task:

```bash
deno task dev run <template-file> --key <invocation-key>
```

**Arguments:**
- `<template-file>` - Path to your template TypeScript file
- `--key <invocation-key>` - Required invocation key for idempotency control

**Example:**

```bash
deno run --allow-net --allow-env --allow-read --allow-write main.ts run ./tmpl/test.ts --key my-unique-key
```

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

Templates are TypeScript files that export a default function receiving a `TemplateContext`:

```typescript
import { TemplateContext } from "../src/template.ts";

export default function (ctx: TemplateContext) {
  // Set template name
  ctx.name("my-template");

  // Set change set name
  ctx.changeSet("my-changeset");

  // Set search strings
  ctx.search(["schema:*", "component:aws"]);

  // Access the invocation key
  const key = ctx.invocationKey();

  // Use the logger
  ctx.logger.info("Template executing");
}
```

### TemplateContext API

**Methods:**

- `name()` - Get the current template name
- `name(newName)` - Set the template name
- `changeSet()` - Get the current change set name
- `changeSet(newName)` - Set the change set name
- `search()` - Get the current search array (defaults to `[]`)
- `search(searchArray)` - Set the search strings
- `invocationKey()` - Get the invocation key (read-only)
- `logger` - Access the logger instance

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
