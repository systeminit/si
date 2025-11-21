# si

A unified command-line tool for managing System Initiative schemas, templates,
and components.

## Features

The `si` CLI provides comprehensive tooling for:

- **AI Agent Integration**: Seamlessly integrate with AI coding tools (Claude
  Code, Cursor, Windsurf)
- **Schema Management**: Author and manage System Initiative schemas locally
- **Template Execution**: Run infrastructure-as-code templates with convergence
- **Component Operations**: Get, update, delete, and search components
- **Remote Sync**: Push and pull schemas from your workspaces
- **Code Generation**: Scaffold schemas and functions with intelligent templates

## Architecture

The `si` CLI is a Deno-based application that provides a structured workflow for
managing System Initiative infrastructure. The architecture consists of several
key components:

### Core Components

- **CLI Module** (`src/cli.ts`): Command-line interface built with Cliffy,
  providing a hierarchical command structure with global options and environment
  variable support
- **Context Module** (`src/context.ts`): Singleton context managing global
  application state, including logging (LogTape) and analytics (PostHog)
  services
- **Project Module** (`src/project.ts`): Project structure management and path
  utilities for working with schemas and their functions
- **Template Engine** (`src/template.ts`): TypeScript-based template execution
  with convergence and idempotency
- **Component APIs** (`src/component/`): Operations for managing components in
  workspaces
- **Authentication** (`src/auth-api-client.ts`, `src/jwt.ts`): API
  authentication and JWT token handling

### Command Structure

```
si
├── ai-agent               - Manage the SI AI Agent (MCP server)
│   ├── init               - Initialize AI agent configuration
│   ├── start              - Launch Claude Code
│   └── config             - Update configuration
├── completion              - Generate shell completions
├── component              - Component-related operations
│   ├── get                - Get component data by name or ID
│   ├── update             - Update component from JSON/YAML file
│   ├── delete             - Delete a component
│   └── search             - Search for components
├── project
│   └── init               - Initialize a new SI project
├── remote
│   └── schema
│       ├── pull           - Pull schemas from remote workspace
│       └── push           - Push schemas to remote workspace
├── run                    - Run a SI template file
├── schema
│   ├── action generate    - Generate action functions
│   ├── authentication generate - Generate authentication functions
│   ├── codegen generate   - Generate code generator functions
│   ├── management generate - Generate management functions
│   ├── qualification generate - Generate qualification functions
│   ├── scaffold generate  - Scaffold a complete schema
│   └── overlay            - Generate overlay functions
├── template
│   └── run                - Run a SI template file (alias)
└── whoami                 - Display authenticated user information
```

### Project Structure

SI projects follow this directory structure:

```
project-root/
├── .siroot                 - Marker file identifying the project root
└── schemas/
    └── <schema-name>/
        ├── .format-version
        ├── schema.ts
        ├── schema.metadata.json
        ├── actions/
        │   ├── create.ts
        │   ├── create.metadata.json
        │   ├── destroy.ts
        │   ├── destroy.metadata.json
        │   ├── refresh.ts
        │   ├── refresh.metadata.json
        │   ├── update.ts
        │   └── update.metadata.json
        ├── codeGenerators/
        │   ├── <codegen-name>.ts
        │   └── <codegen-name>.metadata.json
        ├── management/
        │   ├── <management-name>.ts
        │   └── <management-name>.metadata.json
        └── qualifications/
            ├── <qualification-name>.ts
            └── <qualification-name>.metadata.json
```

## Configuration

### Environment Variables

- `SI_API_TOKEN`: Your System Initiative API token (required for authenticated
  commands)
- `SI_API_BASE_URL`: API endpoint URL (defaults to `https://api.systeminit.com`)
- `SI_BASE_URL`: API base URL for templates (defaults to
  `https://api.systeminit.com`)
- `SI_ROOT`: Project root directory (searches for `.siroot` if not specified)

### Global Options

All commands support these options:

- `--api-token <TOKEN>`: API authentication token
- `--api-base-url <URL>`: Override the API endpoint
- `--root <PATH>`: Specify project root directory
- `-v, --verbose [level]`: Enable verbose logging (0=errors only, 1=+warnings,
  2=+info, 3=+debug, 4=+trace)
- `--no-color`: Disable colored output

## Installation

### Remote Installation (Recommended)

Build and install directly from GitHub without cloning the repository:

```bash
deno compile \
  --allow-all \
  --reload \
  --output=si \
  --import-map=https://raw.githubusercontent.com/systeminit/si/main/bin/si/deno.json \
  https://raw.githubusercontent.com/systeminit/si/main/bin/si/main.ts
```

This downloads the source, compiles it, and creates the `si` executable in the
current directory.

For a specific version or branch, replace `main` with the desired Git reference.

### Local Installation

After building locally (see [Building](#building)), move the executable to a
directory in your PATH:

```bash
# Build locally
deno task build

# Install to user bin directory (Linux/macOS)
mv si ~/.local/bin/

# Or install system-wide (requires sudo)
sudo mv si /usr/local/bin/
```

## Usage

### AI Agent Integration

The `si` CLI includes an AI Agent that integrates System Initiative with Claude
Code through the Model Context Protocol (MCP). Support for other AI tools
(Cursor, Windsurf) is planned for future releases.

#### Quick Start

```bash
# One-time setup
cd /path/to/your/project
si ai-agent init

# Daily use - just one command!
si ai-agent start
# → Launches Claude Code
# → Claude reads .mcp.json and starts bundled MCP server
# → Exit Claude when done (everything stops automatically)
```

#### How It Works

The AI agent is incredibly simple:

1. **Bundled MCP Server**: The MCP server is compiled directly into the `si` binary
2. **Launch Claude**: `si ai-agent start` just launches Claude Code
3. **Claude Manages Everything**: Claude reads `.mcp.json` and spawns the bundled MCP server automatically
4. **Auto-Cleanup**: Exit Claude = MCP server stops automatically

**Single binary. Zero downloads. No process management. Just works.**

#### Initialize the AI Agent

Set up the AI agent for your project:

```bash
si ai-agent init
```

This command:

- Prompts for your System Initiative API token (get one at
  https://auth.systeminit.com/workspaces)
- Saves configuration to `~/.si/ai-agent.json`
- Creates `.mcp.json` in your project (MCP server configuration)
- Creates `.claude/settings.local.json` if using Claude Code

**To get your API token:**

1. Visit https://auth.systeminit.com/workspaces
2. Click the gear icon for your workspace
3. Select "API Tokens"
4. Generate a new token (recommended: 1 year expiration)
5. Copy and paste when prompted

**Options:**

```bash
# Use Claude Code (default, no flag needed)
si ai-agent init

# Provide token via flag (skip prompt)
si ai-agent init --api-token eyJhbGc...

# Specify target directory for config files
si ai-agent init --target-dir /path/to/project

# Future: Other tools (when they support MCP)
# si ai-agent init --tool cursor
# si ai-agent init --tool windsurf
```

#### Start the AI Agent

Launch Claude Code (which automatically starts the bundled MCP server):

```bash
si ai-agent start
```

**What happens:**
```
Starting SI AI Agent...

🚀 Launching Claude Code...
The MCP server will start automatically via .mcp.json

# Claude opens and you start coding...
# Exit Claude when done (Cmd+Q or /exit)
# Everything stops automatically!
```

#### Update Configuration

Change your API token if needed:

```bash
si ai-agent config --update-token
```

#### File Locations

| File                          | Location          | Purpose                                |
| ----------------------------- | ----------------- | -------------------------------------- |
| `ai-agent.json`               | `~/.si/`          | Configuration (token, tool preference) |
| `.mcp.json`                   | Project directory | MCP configuration for Claude Code      |
| `.claude/settings.local.json` | Project directory | Claude-specific settings               |

**Note:** The MCP server is bundled directly in the `si` binary - no separate downloads or binaries needed!

### Schema Management

#### Initialize a Project

Initialize a new SI project with the `project init` command:

```bash
# Initialize in current directory
si project init

# Initialize in a specific directory
si project init /path/to/project

# Or use the --root option
si --root /path/to/project project init
```

This creates a `.siroot` marker file in the project root directory.

#### Create a Schema

Generate a complete schema scaffold:

```bash
si schema scaffold generate MySchema
```

This creates the schema directory structure with template files for the schema
definition and metadata.

#### Generate Functions

Generate specific function types for a schema:

```bash
# Generate an action function
si schema action generate MySchema create

# Generate a code generator
si schema codegen generate MySchema terraform

# Generate a management function
si schema management generate MySchema reconcile

# Generate a qualification
si schema qualification generate MySchema validate

# Generate an authentication function
si schema authentication generate MySchema validate-api-key
```

#### Pull from Remote

Pull schemas from your System Initiative workspace to your local project:

```bash
# Pull a specific schema
si remote schema pull MySchema

# Pull multiple schemas
si remote schema pull Schema1 Schema2 Schema3
```

#### Push to Remote

Push your schemas to your System Initiative workspace:

```bash
si remote schema push
```

### Template Execution

#### Run a Template

Execute a template file to manage infrastructure declaratively:

```bash
SI_API_TOKEN=<your-token> si run <template-file> --key <invocation-key>
```

**Required Arguments:**

- `<template-file>` - Path to your template TypeScript file
- `--key <invocation-key>` - Required invocation key for idempotency control

**Optional Arguments:**

- `-i, --input <file>` - Path to input data file (JSON or YAML)
- `-b, --baseline <file>` - Path to baseline data file (JSON or YAML)
- `-c, --cache-baseline <file>` - Cache baseline results to file
- `--cache-baseline-only` - Exit after writing baseline cache
- `--dry-run` - Show planned changes without executing them

**Example:**

```bash
SI_API_TOKEN=eyJhbGc... si run ./templates/infrastructure.ts --key prod-deploy-001
```

#### Writing Templates

Templates are TypeScript files that export a default function:

```typescript
import { TemplateContext } from "../src/template.ts";
import { z } from "zod";

export default async function (ctx: TemplateContext) {
  // Set template name and change set
  ctx.name("my-infrastructure");
  ctx.changeSet("production");

  // Set search strings to find components
  ctx.search(["schema:AWS EC2 Instance", "tag:production"]);

  // Define input schema with defaults
  ctx.inputs(
    z.object({
      environment: z.string().default("production"),
      replicas: z.number().default(3),
    }),
  );

  // Transform the working set
  ctx.transform((workingSet) => {
    return workingSet.filter((c) => c.name.startsWith("prod-"));
  });

  // Use the logger
  ctx.logger.info("Template executing");
}
```

### Component Operations

#### Get Component

Retrieve detailed information about a component:

```bash
SI_API_TOKEN=<your-token> si component get <component-name-or-id>
```

**Options:**

- `-c, --change-set <id-or-name>` - Specify change set (defaults to HEAD)
- `-o, --output <format>` - Output format: `info` (default), `json`, or `yaml`
- `--cache <file>` - Cache output to file
- `--raw` - Output raw API response as JSON

**Examples:**

```bash
# Get component info
si component get my-server

# Output as YAML and cache to file
si component get my-server --output yaml --cache server.yaml

# Get from specific change set
si component get my-server --change-set dev
```

#### Update Component

Update a component's attributes from a JSON or YAML file:

```bash
SI_API_TOKEN=<your-token> si component update <input-file> --change-set <changeset-name>
```

**Required Options:**

- `-c, --change-set <id-or-name>` - Change set to apply updates in

**Optional Flags:**

- `--component <id-or-name>` - Override component ID from input file
- `--dry-run` - Preview changes without applying them

**Workflow: Get → Edit → Update**

```bash
# 1. Get current component state
si component get my-server --cache server.yaml

# 2. Edit server.yaml with your changes
vim server.yaml

# 3. Apply updates (preview first)
si component update server.yaml --change-set dev --dry-run

# 4. Apply updates
si component update server.yaml --change-set dev
```

#### Delete Component

Delete a component by name or ID:

```bash
SI_API_TOKEN=<your-token> si component delete <component-name-or-id> --change-set <changeset-name>
```

**Options:**

- `-c, --change-set <id-or-name>` - Change set to delete in (required)
- `--dry-run` - Preview deletion without applying

#### Search Components

Search for components using a query:

```bash
SI_API_TOKEN=<your-token> si component search <query>
```

**Options:**

- `-c, --change-set <id-or-name>` - Specify change set (defaults to HEAD)
- `-o, --output <format>` - Output format: `info` (default), `json`, or `yaml`
- `-a, --attribute <path>` - Include specific attribute paths (can be specified
  multiple times)
- `--full-component` - Show full component details for each result

**Examples:**

```bash
# Search by schema
si component search 'schemaName:"AWS EC2 Instance"'

# Search with tags
si component search 'tag:"production" AND tag:"web"'

# Search with specific attributes
si component search 'schemaName:"RDS Database"' --attribute /domain/endpoint --attribute /domain/port
```

### Other Commands

#### Check Authentication

Verify your authentication status:

```bash
si whoami
```

#### Generate Shell Completions

Generate shell completions for bash, zsh, or fish:

```bash
si completion bash > ~/.si-completion.bash
source ~/.si-completion.bash
```

## Development

### Prerequisites

- [Deno](https://deno.land/) runtime (version 1.40+)
- For Buck2 builds: [Buck2](https://buck2.build/) build system

### Running in Development Mode

Run the CLI in development mode with hot reloading:

```bash
deno task dev
```

Without arguments, this displays the help text listing all available commands.

### Running Specific Commands

```bash
# Display help
deno task dev --help

# Generate a schema scaffold
deno task dev schema scaffold generate MySchema

# Run a template
SI_API_TOKEN=<token> deno task dev run ./templates/test.ts --key test-001
```

### Building

#### Building with Deno

Build a standalone executable in the current directory:

```bash
deno task build
```

This creates the `si` executable with all necessary permissions.

#### Building with Buck2

Build using the Buck2 build system:

```bash
buck2 build bin/si
```

For production builds:

```bash
buck2 build //bin/si --mode=release
```

### Testing and Code Quality

#### Running Tests

```bash
deno task test
```

#### Linting

```bash
deno task lint
```

This project uses custom lint rules to enforce code quality. Notably, direct
usage of `Deno.env.get()` is prohibited to ensure proper configuration
management.

#### Formatting

Check code formatting:

```bash
buck2 run //bin/si:check-format
```

Auto-fix formatting issues:

```bash
buck2 run //bin/si:fix-format
```

## Troubleshooting

### Authentication Issues

If you encounter authentication errors:

1. Verify your `SI_API_TOKEN` is set correctly:

   ```bash
   echo $SI_API_TOKEN
   ```

2. Check token validity:

   ```bash
   si whoami
   ```

3. Ensure you're using the correct API endpoint with `--api-base-url` if needed.

### Project Root Not Found

If the CLI cannot find your project root:

1. Initialize a project using `si project init` if you haven't already
2. Verify `.siroot` exists in your project root directory
3. Use the `--root` flag to explicitly specify the project root:
   ```bash
   si --root /path/to/project schema scaffold generate MySchema
   ```

### Verbose Logging

Enable verbose logging to debug issues:

```bash
# Maximum verbosity (trace level)
si -v4 schema scaffold generate MySchema

# Or specify a numeric level (0-4)
si --verbose 4 run template.ts --key test
```

## Code Quality

The project enforces code quality through:

- **Custom Lint Rules**: Prohibits direct usage of `Deno.env.get()` to ensure
  proper configuration management through the Context singleton
- **TypeScript Strict Mode**: Type-safe path handling with specialized path
  classes
- **Structured Logging**: LogTape integration with configurable verbosity levels
- **Comprehensive Testing**: Unit tests for all modules with 296+ test cases

## Contributing

### Code Style

- Follow the existing code structure and TypeScript conventions
- Use the Context singleton for logging and analytics
- Leverage the Project module for path management
- Add JSDoc comments for public APIs
- Run tests and linting before submitting changes

## License

See the root of the System Initiative repository for license information.
