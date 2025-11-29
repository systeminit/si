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
managing System Initiative infrastructure. The codebase follows a **domain-driven
organization** where the folder structure directly mirrors the CLI command hierarchy.

### Core Principles

1. **Command-Code Alignment**: Every CLI command maps directly to its implementation file
   - `si component get` → `src/component/get.ts`
   - `si schema generate` → `src/schema/generate.ts`
   - `si ai-agent init` → `src/ai-agent/init.ts`

2. **Domain Cohesion**: Related code lives together with its domain utilities
   - `src/schema/` - All schema operations + domain utilities (project.ts, generators.ts, materialize.ts, config.ts)
   - `src/component/` - All component operations + domain utilities (cache_api.ts, change_set.ts)
   - `src/template/` - Template execution + template utilities (transpile.ts, helpers.ts)
   - `src/ai-agent/` - AI agent commands + MCP server implementation

3. **Infrastructure Separation**: Cross-cutting concerns organized by purpose
   - `src/cli/` - CLI infrastructure (api.ts, auth.ts, config.ts, jwt.ts)
   - `src/` root - Only global infrastructure (context.ts, logger.ts, si_client.ts)

### Core Components

- **CLI Module** (`src/cli.ts`): Command-line interface built with Cliffy
- **Context Module** (`src/context.ts`): Global application state with logging (LogTape) and analytics (PostHog)
- **Schema Domain** (`src/schema/`): Schema operations, project structure, code generation
- **Component Domain** (`src/component/`): Component CRUD operations with caching and change set management
- **Template Domain** (`src/template/`): TypeScript-based template execution with convergence
- **AI Agent Domain** (`src/ai-agent/`): AI agent configuration and MCP server
- **CLI Infrastructure** (`src/cli/`): Authentication, API context, configuration utilities

### Command Structure

```
si
├── ai-agent                          - Manage the SI AI Agent (MCP server)
│   ├── init                          - Initialize AI agent configuration
│   ├── start                         - Launch Claude Code
│   ├── config                        - Update configuration
│   └── stdio                         - Run MCP server in stdio mode (for external AI tools)
├── completion                         - Generate shell completions
├── component                         - Component-related operations
│   ├── get                           - Get component data by name or ID
│   ├── update                        - Update component from JSON/YAML file
│   ├── delete                        - Delete a component
│   └── search                        - Search for components
├── schema                            - Manage schemas and project
│   ├── init                          - Initialize a new SI project
│   ├── generate                      - Generate schema functions
│   │   ├── action                    - Generate action functions
│   │   ├── authentication            - Generate authentication functions
│   │   ├── codegen                   - Generate code generator functions
│   │   ├── management                - Generate management functions
│   │   ├── qualification             - Generate qualification functions
│   │   └── scaffold                  - Scaffold a complete schema
│   ├── overlay                       - Manage schema overlays
│   │   ├── generate                  - Generate overlay functions
│   │   │   ├── action                - Generate action overlay functions
│   │   │   ├── authentication        - Generate authentication overlay functions
│   │   │   ├── codegen               - Generate codegen overlay functions
│   │   │   ├── management            - Generate management overlay functions
│   │   │   └── qualification         - Generate qualification overlay functions
│   │   └── push                      - Push overlays to workspace
│   ├── pull                          - Pull schemas from workspace
│   └── push                          - Push schemas to workspace
├── template                          - Manage templates
│   └── run                           - Run a SI template file
└── whoami                            - Display authenticated user information
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

1. **Bundled MCP Server**: The MCP server is compiled directly into the `si`
   binary
2. **Launch Claude**: `si ai-agent start` just launches Claude Code
3. **Claude Manages Everything**: Claude reads `.mcp.json` and spawns the
   bundled MCP server automatically
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

**Note:** The MCP server is bundled directly in the `si` binary - no separate
downloads or binaries needed!

### Schema Management

#### Initialize a Project

Initialize a new SI project with the `schema init` command:

```bash
# Initialize in current directory
si schema init

# Initialize in a specific directory
si schema init /path/to/project

# Or use the --root option
si --root /path/to/project schema init
```

This creates a `.siroot` marker file in the project root directory.

#### Create a Schema

Generate a complete schema scaffold:

```bash
si schema generate scaffold MySchema
```

This creates the schema directory structure with template files for the schema
definition and metadata.

#### Generate Functions

Generate specific function types for a schema:

```bash
# Generate an action function
si schema generate action MySchema create

# Generate a code generator
si schema generate codegen MySchema terraform

# Generate a management function
si schema generate management MySchema reconcile

# Generate a qualification
si schema generate qualification MySchema validate

# Generate an authentication function
si schema generate authentication MySchema validate-api-key
```

#### Pull from Remote

Pull schemas from your System Initiative workspace to your local project:

```bash
# Pull a specific schema
si schema pull MySchema

# Pull multiple schemas
si schema pull Schema1 Schema2 Schema3
```

#### Push to Remote

Push your schemas to your System Initiative workspace:

```bash
si schema push
```

### Template Execution

#### Run a Template

Execute a template file to manage infrastructure declaratively:

```bash
SI_API_TOKEN=<your-token> si template run <template-file> --key <invocation-key>
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
SI_API_TOKEN=eyJhbGc... si template run ./templates/infrastructure.ts --key prod-deploy-001
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
deno task dev schema generate scaffold MySchema

# Run a template
SI_API_TOKEN=<token> deno task dev template run ./templates/test.ts --key test-001
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

### Packaging Binary Artifacts

The SI CLI supports packaging into distributable binary artifacts with metadata
sidecars for all supported platforms.

#### Building Artifacts

Build artifacts for specific platforms using platform-specific targets:

```bash
# Build for specific platform (using alias)
buck2 build //bin/si:si-binary-artifact-linux-x86_64
buck2 build //bin/si:si-binary-artifact-darwin-aarch64
buck2 build //bin/si:si-binary-artifact-windows-x86_64

# Build using explicit --target-platforms
buck2 build //bin/si:si-binary-artifact \
  --target-platforms=prelude-si//platforms:linux-x86_64

# Build all platforms
buck2 build \
  //bin/si:si-binary-artifact-linux-x86_64 \
  //bin/si:si-binary-artifact-linux-aarch64 \
  //bin/si:si-binary-artifact-darwin-x86_64 \
  //bin/si:si-binary-artifact-darwin-aarch64 \
  //bin/si:si-binary-artifact-windows-x86_64
```

#### Artifact Structure

- **Unix platforms (Linux, macOS):** `.tar.gz` archives with flat structure
- **Windows platforms:** `.zip` archives with flat structure
- Each archive contains the binary and a `metadata.json` sidecar file

#### Supported Platforms

- `linux-x86_64`, `linux-aarch64`
- `darwin-x86_64` (Intel Macs), `darwin-aarch64` (Apple Silicon)
- `windows-x86_64`

**Note:** Windows ARM64 (`windows-aarch64`) is not supported because Deno does
not provide a compilation target for that platform.

For detailed information on artifact packaging, including publishing and
promoting artifacts, see `docs/build/deno-binary-artifacts.md`.

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

1. Initialize a project using `si schema init` if you haven't already
2. Verify `.siroot` exists in your project root directory
3. Use the `--root` flag to explicitly specify the project root:
   ```bash
   si --root /path/to/project schema generate scaffold MySchema
   ```

### Verbose Logging

Enable verbose logging to debug issues:

```bash
# Maximum verbosity (trace level)
si -v4 schema generate scaffold MySchema

# Or specify a numeric level (0-4)
si --verbose 4 template run template.ts --key test
```

## Code Quality

The project enforces code quality through:

- **Custom Lint Rules**: Prohibits direct usage of `Deno.env.get()` to ensure
  proper configuration management through the Context singleton
- **TypeScript Strict Mode**: Type-safe path handling with specialized path
  classes
- **Structured Logging**: LogTape integration with configurable verbosity levels
- **Comprehensive Testing**: Unit tests for all modules with 296+ test cases

## Adding New Commands

The codebase follows a consistent pattern for adding new CLI commands. The folder structure
directly mirrors the command hierarchy, making it intuitive to add new functionality.

### Pattern for New Commands

#### 1. Determine Command Location

Map your CLI command to a file path:
- `si mycommand run` → `src/mycommand/run.ts`
- `si mycommand subcommand action` → `src/mycommand/subcommand/action.ts`

#### 2. Create Implementation File

Create your command file with a `call*` function that contains the full implementation:

```typescript
// src/mycommand/run.ts
import type { Context } from "../context.ts";

export interface MyCommandRunOptions {
  flag?: string;
  // ... other options
}

/**
 * Execute the mycommand run command
 */
export async function callMyCommandRun(
  ctx: Context,
  options: MyCommandRunOptions,
): Promise<void> {
  // Full command implementation here
  ctx.logger.info("Running my command");
  // ... implementation
}
```

#### 3. Add Domain Utilities (Optional)

If your command needs domain-specific utilities, place them in the same folder:

```
src/mycommand/
├── run.ts           # Command implementation
├── config.ts        # Domain configuration
├── helpers.ts       # Domain-specific helpers
└── cache.ts         # Domain-specific caching
```

**Key Rule**: Domain utilities should only be used by that domain. If a utility is needed
by multiple domains, it belongs at `src/` root or in `src/cli/` for CLI infrastructure.

#### 4. Register Command in CLI

Update `src/cli.ts` to register your command:

```typescript
// Add import at top
import { callMyCommandRun, type MyCommandRunOptions } from "./mycommand/run.ts";

// Add command registration in buildProgram()
function buildProgram() {
  return new Command()
    // ... existing commands
    .command(
      "mycommand",
      new Command()
        .description("Description of mycommand")
        .command(
          "run",
          new Command()
            .description("Run my command")
            .option("-f, --flag <value>", "Flag description")
            .action(async (options) => {
              const ctx = Context.instance();
              await callMyCommandRun(ctx, options as MyCommandRunOptions);
            })
        )
    );
}
```

#### 5. Determine Where Utilities Belong

**Place utilities in domain folder (`src/mycommand/`) if:**
- Only used by this command/domain
- Example: Template-specific helpers in `src/template/helpers.ts`

**Place utilities in CLI infrastructure (`src/cli/`) if:**
- Used for CLI setup, authentication, or configuration
- Example: JWT utilities in `src/cli/jwt.ts`

**Place utilities at root (`src/`) if:**
- Used across ALL domains (component, schema, template, etc.)
- Example: `si_client.ts` (global API config), `logger.ts` (global logging)

### Examples from Codebase

**Simple command with no subcommands:**
- Command: `si whoami`
- Location: `src/whoami.ts`
- Pattern: Single file at root for top-level commands

**Command group with multiple operations:**
- Commands: `si component get`, `si component update`, etc.
- Location: `src/component/get.ts`, `src/component/update.ts`, etc.
- Utilities: `src/component/cache_api.ts`, `src/component/change_set.ts`
- Pattern: Folder per command group with operations as files

**Nested command hierarchy:**
- Command: `si schema generate action`
- Location: `src/schema/generate.ts` (contains `callSchemaFuncGenerate`)
- Pattern: Single file handling multiple related subcommands

### Anti-Patterns to Avoid

❌ **Don't create thin wrapper files** - The command file IS the implementation, not a wrapper

❌ **Don't put domain code at root** - Domain-specific utilities belong in domain folders

❌ **Don't mix domain utilities** - `config.ts` at root shouldn't contain both schema and CLI config

## Contributing

### Code Style

- Follow the existing code structure and TypeScript conventions
- Use the Context singleton for logging and analytics
- Place code according to the domain-driven organization principles
- Add JSDoc comments for public APIs
- Run tests and linting before submitting changes

### Outstanding Development Tasks

- [ ] Remove all temporary exemptions of `Deno.env.get()` and replace with
      configuration injection.
- [ ] Use consistent and cross-platform path computation for configuration file
      paths, following `XDG_CONFIG_HOME` standards on Unix platforms
      (`ai_agent.ts` module).
- [ ] Extract command logic in `mcp-server` subcommand into subcommand,
      consistent with other subcommands.
- [ ] Remove locally duplicated `--api-token` option in `ai-agent init`
      subcommand which is already a global option.
- [ ] Split hidden subcommands into explicit ones which are represented by
      `--show`/`--update-token`/`--tool` in `ai-agent config` subcommand.
- [ ] Remove locally duplicated `SI_API_TOKEN` and `SI_BASE_URL` environment
      variable parsing in the `run` subcommand as they are already global
      options (both as options and environment variables).
- [ ] Remove locally duplicated `SI_API_TOKEN` and `SI_BASE_URL` environment
      variable parsing in the `template run` subcommand as they are already
      global options (both as options and environment variables).
- [ ] Provide an alternative mechansim (likely a Cliffy command alias) to avoid
      duplicate configuration of the `run` and `template run` subcommands.
- [ ] Remove locally duplicated `SI_API_TOKEN` and `SI_BASE_URL` environment
      variable parsing in the `component get` subcommand as they are already
      global options (both as options and environment variables).
- [ ] Remove locally duplicated `SI_API_TOKEN` and `SI_BASE_URL` environment
      variable parsing in the `component update` subcommand as they are already
      global options (both as options and environment variables).
- [ ] Remove locally duplicated `SI_API_TOKEN` and `SI_BASE_URL` environment
      variable parsing in the `component delete` subcommand as they are already
      global options (both as options and environment variables).
- [ ] Remove locally duplicated `SI_API_TOKEN` and `SI_BASE_URL` environment
      variable parsing in the `component search` subcommand as they are already
      global options (both as options and environment variables).
- [ ] Consider a common short option character for `--change-set` options across
      CLI (the `-c` is used inconsistently and may lead to confusion when
      calling other subcommands with short options).

## License

See the root of the System Initiative repository for license information.
