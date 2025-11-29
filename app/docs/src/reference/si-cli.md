---
outline: [2, 3, 4]
---

# SI CLI

Updated November 28, 2025.

The `si` CLI is a unified command-line tool for managing System Initiative
schemas, templates, and components. It provides comprehensive tooling for schema
management, template execution, component operations, and AI agent integration.

## Overview

The `si` CLI allows you to:

- **Manage Schemas**: Author and manage System Initiative schemas locally,
  generate functions, and sync with remote workspaces
- **Execute Templates**: Run infrastructure-as-code templates with convergence
  and idempotency
- **Operate on Components**: Get, update, delete, and search components
  programmatically
- **Integrate AI Agents**: Set up and manage AI coding assistants (Claude Code,
  Cursor, Windsurf) with the bundled MCP server
- **Sync with Workspaces**: Push and pull schemas and overlays from your remote
  workspaces

## Installation

### Quick Install (Recommended)

The easiest way to install the SI CLI is using our installation script:

#### Linux & macOS

**Basic Installation:**

```bash
# Install for current user (recommended)
curl -fsSL https://auth.systeminit.com/install.sh | sh
```

This installs to your user directory (`$HOME/.local/bin` or `$HOME/bin`). No sudo required!

**System-wide Installation:**

```bash
# Install for all users (requires sudo)
curl -fsSL https://auth.systeminit.com/install.sh | sudo sh
```

This installs to `/usr/local/bin`, making it available to all users on the system.

:::tip
For most users, the user installation is recommended. It doesn't require admin privileges and works perfectly for personal development.
:::

**What the script does:**
- Automatically detects your platform (Linux/macOS) and architecture (x86_64/aarch64)
- Downloads the latest stable release
- Extracts and installs the binary with correct permissions
- On macOS: automatically removes quarantine attributes so the binary runs without security prompts
- Intelligently selects installation directory based on your PATH and whether you use sudo

**Installation Options:**

```bash
# Install specific version
curl -fsSL https://auth.systeminit.com/install.sh | sh -s -- -V stable

# Install to custom location
curl -fsSL https://auth.systeminit.com/install.sh | sh -s -- -d ~/.local/bin

# Install specific platform
curl -fsSL https://auth.systeminit.com/install.sh | sh -s -- -p darwin-aarch64

# See all options
curl -fsSL https://auth.systeminit.com/install.sh | sh -s -- --help
```

:::info
The installation script supports:
- **Linux**: x86_64, aarch64
- **macOS**: x86_64 (Intel), aarch64 (Apple Silicon)

For Windows, see the Windows installation instructions below.
:::

#### Windows

**Quick Install (PowerShell):**

```powershell
# Run the installation script
irm https://auth.systeminit.com/install.ps1 | iex
```

The script will:
- Automatically detect your system architecture (x86_64)
- Download the latest stable release
- Extract and install the binary
- Add to your PATH automatically
- Install to `$env:LOCALAPPDATA\si` (user install) or `C:\Program Files\si` (admin)

**Run as Administrator** (optional, for system-wide install):
```powershell
# Right-click PowerShell and select "Run as Administrator", then:
irm https://auth.systeminit.com/install.ps1 | iex
```

:::tip
For most users, the regular (non-admin) installation works great. It installs to your user directory and doesn't require administrator privileges.
:::

:::warning PowerShell Alias Conflict
PowerShell has a built-in alias `si` for the `Set-Item` cmdlet. After installation, use `si.exe` to run commands:

```powershell
si.exe --version
si.exe --help
```

Alternatively, remove the alias by adding this to your PowerShell profile (`$PROFILE`):
```powershell
Remove-Item alias:si -Force -ErrorAction SilentlyContinue
```
:::

**Installation Options:**

```powershell
# Download and run with options
Invoke-WebRequest https://auth.systeminit.com/install.ps1 -OutFile install.ps1
.\install.ps1 -Help
.\install.ps1 -Destination "C:\Tools\si"
.\install.ps1 -Version "stable"
```

### Manual Installation

If you prefer to download and install manually:

<details>
<summary>Linux (aarch64)</summary>

```bash
curl -LO https://artifacts.systeminit.com/si/stable/binary/linux/aarch64/si-stable-binary-linux-aarch64.tar.gz
tar -xzf si-stable-binary-linux-aarch64.tar.gz
sudo mv si /usr/local/bin/
# Or move to user bin: mv si ~/.local/bin/
```
</details>

<details>
<summary>Linux (x86_64)</summary>

```bash
curl -LO https://artifacts.systeminit.com/si/stable/binary/linux/x86_64/si-stable-binary-linux-x86_64.tar.gz
tar -xzf si-stable-binary-linux-x86_64.tar.gz
sudo mv si /usr/local/bin/
# Or move to user bin: mv si ~/.local/bin/
```
</details>

<details>
<summary>macOS (Apple Silicon)</summary>

```bash
curl -LO https://artifacts.systeminit.com/si/stable/binary/darwin/aarch64/si-stable-binary-darwin-aarch64.tar.gz
tar -xzf si-stable-binary-darwin-aarch64.tar.gz
sudo mv si /usr/local/bin/
# Or move to user bin: mv si ~/.local/bin/

# Remove quarantine attribute
xattr -d com.apple.quarantine /usr/local/bin/si
```
</details>

<details>
<summary>macOS (Intel)</summary>

```bash
curl -LO https://artifacts.systeminit.com/si/stable/binary/darwin/x86_64/si-stable-binary-darwin-x86_64.tar.gz
tar -xzf si-stable-binary-darwin-x86_64.tar.gz
sudo mv si /usr/local/bin/
# Or move to user bin: mv si ~/.local/bin/

# Remove quarantine attribute
xattr -d com.apple.quarantine /usr/local/bin/si
```
</details>

<details>
<summary>Windows (x86_64)</summary>

**Using PowerShell:**

```powershell
# Download the binary
Invoke-WebRequest -Uri https://artifacts.systeminit.com/si/stable/binary/windows/x86_64/si-stable-binary-windows-x86_64.zip -OutFile si.zip

# Extract the binary
Expand-Archive -Path si.zip -DestinationPath .

# Move to a directory in your PATH (example: C:\Program Files\si)
New-Item -ItemType Directory -Force -Path "C:\Program Files\si"
Move-Item -Path .\si.exe -Destination "C:\Program Files\si\si.exe"

# Add to PATH (requires admin privileges)
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";C:\Program Files\si", [EnvironmentVariableTarget]::Machine)
```

**Using Command Prompt (cmd):**

```cmd
REM Download the binary (requires curl, available in Windows 10+)
curl -LO https://artifacts.systeminit.com/si/stable/binary/windows/x86_64/si-stable-binary-windows-x86_64.zip

REM Extract using tar (available in Windows 10+)
tar -xf si-stable-binary-windows-x86_64.zip

REM Move to a directory in your PATH
move si.exe "C:\Program Files\si\si.exe"

REM Add to PATH (requires admin privileges)
setx /M PATH "%PATH%;C:\Program Files\si"
```
</details>

## Global Options

All commands support these global options:

| Option                  | Type   | Description                                        | Default                               |
| ----------------------- | ------ | -------------------------------------------------- | ------------------------------------- |
| `--api-token <TOKEN>`   | string | Your System Initiative API token                   | -                                     |
| `--api-base-url <URL>`  | string | API endpoint URL                                   | `https://api.systeminit.com`          |
| `--root <PATH>`         | string | Project root directory (rarely needed - see below) | Auto-discovers from current directory |
| `-v, --verbose [level]` | number | Verbose logging (0-4)                              | `2`                                   |
| `--no-color`            | flag   | Disable colored output                             | -                                     |
| `-h, --help`            | flag   | Show help                                          | -                                     |
| `-V, --version`         | flag   | Show version                                       | -                                     |

#### Project Root Discovery

By default, the CLI automatically finds your project root by searching upward
from your current directory for a `.siroot` marker file. This means you can run
`si` commands from anywhere inside your project - just like Git.

The `--root` option is only needed in these scenarios:

- Scripts that need to explicitly specify a project
- CI/CD environments
- Working with multiple projects simultaneously
- When you're outside a project and want to target a specific one

**Example of automatic discovery:**

```bash
# Works from anywhere in your project
cd /path/to/project/schemas/AWS::EC2::Instance
si schema generate action "AWS::EC2::Instance" create  # ← Finds .siroot automatically

# Or from project root
cd /path/to/project
si schema generate action "AWS::EC2::Instance" create
```

**Example with explicit root:**

```bash
# Useful for scripts or when outside the project
si --root /path/to/project schema generate action "AWS::EC2::Instance" create
```

### Verbosity Levels

- `0` - Errors only
- `1` - Errors + Warnings
- `2` - Errors + Warnings + Info (default)
- `3` - Errors + Warnings + Info + Debug
- `4` - Errors + Warnings + Info + Debug + Trace

## Environment Variables

| Variable          | Description                                                        |
| ----------------- | ------------------------------------------------------------------ |
| `SI_API_TOKEN`    | System Initiative API token (required for authenticated commands)  |
| `SI_API_BASE_URL` | API endpoint URL                                                   |
| `SI_BASE_URL`     | API base URL for templates                                         |
| `SI_ROOT`         | Project root directory (rarely needed - auto-discovers by default) |

### Setting Up Authentication

Most `si` commands require authentication via the `SI_API_TOKEN` environment
variable. Rather than prefixing every command with `SI_API_TOKEN=...`, we
recommend setting it in your shell session or profile:

**For a single session (Linux/macOS):**

```bash
# Set for current terminal session
export SI_API_TOKEN=your_token_here

# Now all commands work without the prefix
si whoami
si schema pull "*"
si component search 'schemaName:"AWS EC2 Instance"'
```

**For a single session (Windows PowerShell):**

```powershell
# Set for current PowerShell session
$env:SI_API_TOKEN="your_token_here"

# Now all commands work without the prefix
si whoami
si schema pull "*"
si component search 'schemaName:"AWS EC2 Instance"'
```

**For a single session (Windows Command Prompt):**

```cmd
REM Set for current cmd session
set SI_API_TOKEN=your_token_here

REM Now all commands work without the prefix
si whoami
si schema pull "*"
si component search "schemaName:\"AWS EC2 Instance\""
```

**Permanently in your shell profile (Linux/macOS):**

```bash
# Add to ~/.bashrc, ~/.zshrc, or ~/.config/fish/config.fish
echo 'export SI_API_TOKEN=your_token_here' >> ~/.bashrc
source ~/.bashrc
```

**Permanently (Windows PowerShell):**

```powershell
# Set permanently for current user
[Environment]::SetEnvironmentVariable("SI_API_TOKEN", "your_token_here", [EnvironmentVariableTarget]::User)

# Restart PowerShell for changes to take effect
```

**Permanently (Windows Command Prompt):**

```cmd
REM Set permanently for current user
setx SI_API_TOKEN "your_token_here"

REM Restart Command Prompt for changes to take effect
```

**For multiple workspaces:**

If you work with multiple workspaces, consider using direnv (Linux/macOS) or similar tools to
set different tokens per project directory.

:::tip
Like the `--root` option, the `SI_ROOT` environment variable is rarely
needed. The CLI automatically discovers your project root by searching upward
for `.siroot`.
:::

## Commands

:::tip
Authentication Required Most commands below require the `SI_API_TOKEN`
environment variable to be set. See the
[Setting Up Authentication](#setting-up-authentication) section above for
details on how to configure this.
:::

### `whoami` - Verify Authentication

Displays authenticated user information.

#### Usage

```bash
si whoami
```

#### Example

```bash
# Check authentication status
si whoami
```

---

### `completion` - Generate Shell Completions

Generate shell completion scripts for bash, fish, or zsh.

#### Usage

```bash
si completion <shell>
```

#### Available Shells

- `bash` - Generate completions for bash
- `fish` - Generate completions for fish
- `zsh` - Generate completions for zsh

#### Examples

```bash
# Bash completion
si completion bash > ~/.si-completion.bash
echo "source ~/.si-completion.bash" >> ~/.bashrc

# Zsh completion
si completion zsh > ~/.si-completion.zsh
echo "source ~/.si-completion.zsh" >> ~/.zshrc

# Fish completion
si completion fish > ~/.config/fish/completions/si.fish
```

---

## AI Agent Integration

The `si` CLI includes a bundled AI Agent (MCP server) that integrates System
Initiative with AI coding tools like Claude Code through the Model Context
Protocol (MCP).

### Quick Start

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

### `ai-agent init` - Initialize AI Agent

One-time setup: configure token and create MCP files.

#### Usage

```bash
si ai-agent init [OPTIONS]
```

#### Options

| Option                | Type   | Description                      | Default                 |
| --------------------- | ------ | -------------------------------- | ----------------------- |
| `--target-dir <path>` | string | Directory to create config files | Current directory       |
| `--api-token <token>` | string | System Initiative API token      | Prompts if not provided |
| `--tool <name>`       | string | AI tool: `claude`                | `claude`                |

#### What It Does

1. Prompts for your System Initiative API token
2. Creates `.mcp.json` in your project (MCP server configuration)
3. Creates `.claude/settings.local.json` if using Claude Code

#### Getting Your API Token

1. Visit
   [auth.systeminit.com/workspaces](https://auth.systeminit.com/workspaces)
2. Click the gear icon for your workspace
3. Select "API Tokens"
4. Generate a new token
5. Copy and paste when prompted

#### Examples

```bash
# Interactive setup (prompts for token)
cd /path/to/your/project
si ai-agent init

# Provide token via flag - this will skip the user being prompted for the token
si ai-agent init --api-token eyJhbGc...

# Specify target directory
si ai-agent init --target-dir /path/to/project

# Use different AI tool
si ai-agent init --tool cursor
```

---

### `ai-agent start` - Launch AI Tool

Launch Claude Code (which automatically starts the bundled MCP server).

#### Usage

```bash
si ai-agent start [OPTIONS]
```

#### Options

| Option          | Type   | Description       | Default  |
| --------------- | ------ | ----------------- | -------- |
| `--tool <name>` | string | AI tool to launch | `claude` |

#### Examples

```bash
# Launch Claude Code (default)
si ai-agent start

# Launch specific tool
si ai-agent start --tool claude
```

---

### `ai-agent config` - Manage Configuration

View or update AI agent configuration.

#### Usage

```bash
si ai-agent config [OPTIONS]
```

#### Options

| Option           | Type   | Description                          |
| ---------------- | ------ | ------------------------------------ |
| `--show`         | flag   | Show current configuration (default) |
| `--update-token` | flag   | Update the API token                 |
| `--tool <name>`  | string | Update AI tool: `claude`             |

#### Examples

```bash
# Show current configuration
si ai-agent config
si ai-agent config --show

# Update API token
si ai-agent config --update-token
```

---

### `ai-agent stdio` - Run MCP Server

Run MCP server in stdio mode (for external AI tools to connect).

This command runs the MCP server in stdio mode, which is used internally by AI
tools that read the `.mcp.json` configuration. You typically don't need to run
this manually - use `si ai-agent start` instead.

#### Usage

```bash
si ai-agent stdio
```

#### Example

```bash
# Run MCP server in stdio mode
si ai-agent stdio
```

---

## Schema Management

Manage schemas: initialize projects, generate functions locally, pull from and
push to remote workspaces.

### `schema init` - Initialize Project

Initialize a new System Initiative project by creating a `.siroot` marker file.

#### Usage

```bash
si schema init [ROOT_PATH]
```

#### Arguments

| Argument    | Type   | Description             | Default           |
| ----------- | ------ | ----------------------- | ----------------- |
| `ROOT_PATH` | string | Directory to initialize | Current directory |

#### What It Does

Creates a `.siroot` marker file in the specified directory (or current directory
if not specified). This marker file identifies the project root and enables the
CLI to automatically discover your project from any subdirectory.

:::tip
After initializing, you can run `si` commands from anywhere within your
project. The CLI will automatically find the `.siroot` file by searching upward
through parent directories.
:::

#### Examples

```bash
# Initialize in current directory (most common)
cd /path/to/my-project
si schema init

# Initialize a specific directory
si schema init /path/to/project

# After initialization, these both work from anywhere in the project:
cd /path/to/my-project/schemas/AWS::EC2::Instance
si schema generate action "AWS::EC2::Instance" create  # ← Works!

cd /path/to/my-project
si schema generate action "AWS::EC2::Instance" create  # ← Also works!
```

---

### `schema generate scaffold` - Scaffold Complete Schema

Scaffolds a complete schema with all default functions and metadata.

#### Usage

```bash
si schema generate scaffold [SCHEMA_NAME]
```

#### Arguments

| Argument      | Type   | Description                    |
| ------------- | ------ | ------------------------------ |
| `SCHEMA_NAME` | string | Name of the schema to scaffold |

#### What It Creates

```
schemas/
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
    ├── management/
    └── qualifications/
```

#### Examples

```bash
# Scaffold a new schema
si schema generate scaffold "AWS::EC2::Instance"

# Scaffold with explicit root
si --root /path/to/project schema generate scaffold "AWS::EC2::SecurityGroup"
```

---

### `schema generate action` - Generate Action Functions

Generates action functions (create, destroy, refresh, update) for schemas.

#### Usage

```bash
si schema generate action [SCHEMA_NAME] [ACTION_NAME]
```

#### Arguments

| Argument      | Type   | Description        | Options                                  |
| ------------- | ------ | ------------------ | ---------------------------------------- |
| `SCHEMA_NAME` | string | Name of the schema | -                                        |
| `ACTION_NAME` | string | Name of the action | `create`, `destroy`, `refresh`, `update` |

#### Examples

```bash
# Generate a create action
si schema generate action "AWS::EC2::Instance" create

# Generate all action functions
si schema generate action "AWS::EC2::Instance" create
si schema generate action "AWS::EC2::Instance" destroy
si schema generate action "AWS::EC2::Instance" refresh
si schema generate action "AWS::EC2::Instance" update
```

---

### `schema generate authentication` - Generate Authentication Functions

Generates authentication functions for schemas.

#### Usage

```bash
si schema generate authentication [SCHEMA_NAME] [AUTH_NAME]
```

#### Arguments

| Argument      | Type   | Description                         |
| ------------- | ------ | ----------------------------------- |
| `SCHEMA_NAME` | string | Name of the schema                  |
| `AUTH_NAME`   | string | Name of the authentication function |

#### Examples

```bash
# Generate an authentication function
si schema generate authentication "AWS Credential" validate-credentials

# Generate OAuth authentication
si schema generate authentication "GitHub Credential" oauth2
```

---

### `schema generate codegen` - Generate Code Generator Functions

Generates code generator functions for schemas.

#### Usage

```bash
si schema generate codegen [SCHEMA_NAME] [CODEGEN_NAME]
```

#### Arguments

| Argument       | Type   | Description                |
| -------------- | ------ | -------------------------- |
| `SCHEMA_NAME`  | string | Name of the schema         |
| `CODEGEN_NAME` | string | Name of the code generator |

#### Examples

```bash
# Generate a Terraform code generator
si schema generate codegen "AWS::EC2::Instance" terraform

# Generate a CloudFormation code generator
si schema generate codegen "AWS::EC2::VPC" cloudformation

# Generate a Pulumi code generator
si schema generate codegen "AWS::EC2::SecurityGroup" pulumi
```

---

### `schema generate management` - Generate Management Functions

Generates management functions for schemas.

#### Usage

```bash
si schema generate management [SCHEMA_NAME] [MANAGEMENT_NAME]
```

#### Arguments

| Argument          | Type   | Description                     |
| ----------------- | ------ | ------------------------------- |
| `SCHEMA_NAME`     | string | Name of the schema              |
| `MANAGEMENT_NAME` | string | Name of the management function |

#### Examples

```bash
# Generate a reconcile function
si schema generate management "AWS::EC2::Instance" reconcile

# Generate a status check function
si schema generate management "AWS::EC2::Instance" check-status
```

---

### `schema generate qualification` - Generate Qualification Functions

Generates qualification functions for schemas.

#### Usage

```bash
si schema generate qualification [SCHEMA_NAME] [QUALIFICATION_NAME]
```

#### Arguments

| Argument             | Type   | Description                        |
| -------------------- | ------ | ---------------------------------- |
| `SCHEMA_NAME`        | string | Name of the schema                 |
| `QUALIFICATION_NAME` | string | Name of the qualification function |

#### Examples

```bash
# Generate a validation qualification
si schema generate qualification "AWS::EC2::SecurityGroup" validate

# Generate a compliance check qualification
si schema generate qualification "AWS::EC2::SecurityGroup" compliance-check
```

---

### `schema pull` - Pull from Remote

Pulls schemas from your remote System Initiative workspace.

#### Usage

```bash
si schema pull [SCHEMA_NAME...]
```

#### Arguments

| Argument      | Type   | Description                               |
| ------------- | ------ | ----------------------------------------- |
| `SCHEMA_NAME` | string | Schema names to pull (supports wildcards) |

#### Options

| Option       | Type | Description             | Default            |
| ------------ | ---- | ----------------------- | ------------------ |
| `--builtins` | flag | Include builtin schemas | Skipped by default |

#### Wildcard Patterns

- `*` - Pull all schemas
- `AWS::*` - Pull all AWS schemas
- `Fastly::*` - Pull all Fastly schemas

#### Examples

```bash
# Pull a specific schema
si schema pull "AWS::EC2::Instance"

# Pull multiple schemas
si schema pull "AWS::EC2::Instance" "AWS::EC2::VPC" "AWS::EC2::Subnet"

# Pull all schemas in a category
si schema pull "AWS::EC2::*"

# Pull all schemas
si schema pull "*"

# Include builtin schemas
si schema pull --builtins "*"
```

---

### `schema push` - Push to Remote

Pushes schemas to your remote System Initiative workspace.

#### Usage

```bash
si schema push [SCHEMA_NAME...]
```

#### Arguments

| Argument      | Type   | Description          |
| ------------- | ------ | -------------------- |
| `SCHEMA_NAME` | string | Schema names to push |

#### Options

| Option                    | Type | Description                            | Default      |
| ------------------------- | ---- | -------------------------------------- | ------------ |
| `-s, --skip-confirmation` | flag | Skip confirmation prompt               | Shows prompt |
| `-b, --update-builtins`   | flag | Change builtin schemas (SI Admin Only) | -            |

#### Examples

```bash
# Push all schemas (prompts for confirmation)
si schema push

# Push specific schema
si schema push "AWS::EC2::Instance"

# Push multiple schemas
si schema push "AWS::EC2::Instance" "AWS::EC2::VPC" "AWS::EC2::Subnet"

# Skip confirmation prompt
si schema push --skip-confirmation

# Admin: Update builtin schemas
si schema push --update-builtins "AWS::EC2::Instance"
```

---

### Schema Overlays

Manage schema overlays: generate overlay functions and push them to remote
workspaces. Overlays allow you to customize or extend existing schemas without
modifying the original schema definition.

#### `schema overlay generate` - Generate Overlay Functions

Generate overlay function definitions that customize or extend existing schemas.

##### Subcommands

- `action` - Generate action overlay functions
- `authentication` - Generate authentication overlay functions
- `codegen` - Generate codegen overlay functions
- `management` - Generate management overlay functions
- `qualification` - Generate qualification overlay functions

##### Examples

```bash
# Generate action overlay
si schema overlay generate action "AWS::EC2::Instance" custom-terminate

# Generate authentication overlay
si schema overlay generate authentication "AWS Credential" custom-auth

# Generate codegen overlay
si schema overlay generate codegen "AWS::EC2::Instance" custom-terraform
```

---

#### `schema overlay push` - Push Overlays

Pushes overlay funcs to your remote System Initiative workspace.

##### Usage

```bash
si schema overlay push [SCHEMA_NAME...]
```

##### Arguments

| Argument      | Type   | Description                     |
| ------------- | ------ | ------------------------------- |
| `SCHEMA_NAME` | string | Schema names (overlays) to push |

##### Examples

```bash
# Push specific overlay
si schema overlay push "AWS::EC2::Instance"

# Push multiple overlays
si schema overlay push "AWS::EC2::Instance" "AWS::EC2::VPC" "AWS::EC2::SecurityGroup"

# Push all overlays (if no schema names specified)
si schema overlay push
```

---

## Component Operations

Component-related operations: get, update, delete, and search components.

### `component get` - Get Component

Get component data by name or ID.

#### Usage

```bash
si component get <component> [OPTIONS]
```

#### Arguments

| Argument    | Type   | Description          |
| ----------- | ------ | -------------------- |
| `component` | string | Component name or ID |

#### Options

| Option                  | Type   | Description                                  | Default |
| ----------------------- | ------ | -------------------------------------------- | ------- |
| `-c, --change-set <id>` | string | Change set ID or name                        | `HEAD`  |
| `-o, --output <format>` | string | Output format: `info`, `json`, or `yaml`     | `info`  |
| `--cache <file>`        | string | Cache output to file (format from extension) | -       |
| `--raw`                 | flag   | Output raw API response as JSON              | -       |

#### Environment Variables

- `SI_API_TOKEN` (required)
- `SI_BASE_URL`

#### Examples

```bash
# Get component info
si component get my-server

# Output as JSON
si component get my-server --output json

# Output as YAML and cache to file
si component get my-server --output yaml --cache server.yaml

# Get from specific change set
si component get my-server --change-set dev

# Get raw API response
si component get my-server --raw
```

---

### `component update` - Update Component

Update a component from JSON/YAML file (idempotent).

#### Usage

```bash
si component update <input-file> --change-set <id-or-name> [OPTIONS]
```

#### Arguments

| Argument     | Type   | Description                       |
| ------------ | ------ | --------------------------------- |
| `input-file` | string | Path to input file (JSON or YAML) |

#### Options

| Option                          | Type   | Description                        | Required |
| ------------------------------- | ------ | ---------------------------------- | -------- |
| `-c, --change-set <id-or-name>` | string | Change set ID or name              | Yes      |
| `--component <id-or-name>`      | string | Override component ID from file    | No       |
| `--dry-run`                     | flag   | Show diff without applying changes | No       |

#### Environment Variables

- `SI_API_TOKEN` (required)
- `SI_BASE_URL`

#### Workflow: Get → Edit → Update

```bash
# 1. Get current component state
si component get my-server --cache server.yaml

# 2. Edit server.yaml with your changes
vim server.yaml

# 3. Preview changes
si component update server.yaml --change-set dev --dry-run

# 4. Apply updates
si component update server.yaml --change-set dev
```

#### Examples

```bash
# Update component from YAML file
si component update server.yaml --change-set dev

# Update with component ID override
si component update config.json --change-set dev --component my-server-id

# Preview changes first
si component update server.yaml --change-set dev --dry-run
```

---

### `component delete` - Delete Component

Delete a component by name or ID.

#### Usage

```bash
si component delete <component> --change-set <id-or-name> [OPTIONS]
```

#### Arguments

| Argument    | Type   | Description          |
| ----------- | ------ | -------------------- |
| `component` | string | Component name or ID |

#### Options

| Option                          | Type   | Description                       | Required |
| ------------------------------- | ------ | --------------------------------- | -------- |
| `-c, --change-set <id-or-name>` | string | Change set ID or name             | Yes      |
| `--dry-run`                     | flag   | Preview deletion without applying | No       |

#### Environment Variables

- `SI_API_TOKEN` (required)
- `SI_BASE_URL`

#### Examples

```bash
# Delete a component
si component delete my-server --change-set dev

# Preview deletion first
si component delete my-server --change-set dev --dry-run

# Delete by component ID
si component delete comp_1234567890 --change-set dev
```

---

### `component search` - Search Components

Search for components using a search query.

#### Usage

```bash
si component search <query> [OPTIONS]
```

#### Arguments

| Argument | Type   | Description  |
| -------- | ------ | ------------ |
| `query`  | string | Search query |

#### Options

| Option                   | Type   | Description                                   | Default |
| ------------------------ | ------ | --------------------------------------------- | ------- |
| `-c, --change-set <id>`  | string | Change set ID or name                         | `HEAD`  |
| `-o, --output <format>`  | string | Output format: `info`, `json`, or `yaml`      | `info`  |
| `-a, --attribute <path>` | string | Include specific attribute paths (repeatable) | -       |
| `--full-component`       | flag   | Show full component details                   | -       |

#### Environment Variables

- `SI_API_TOKEN` (required)
- `SI_BASE_URL`

#### Search Query Syntax

```
schemaName:"Schema Name"     - Search by schema name
tag:"tag-name"               - Search by tag
AND                          - Combine conditions
OR                           - Alternative conditions
```

#### Examples

```bash
# Search by schema
si component search 'schemaName:"AWS EC2 Instance"'

# Search with tags
si component search 'tag:"production" AND tag:"web"'

# Search with specific attributes
si component search 'schemaName:"RDS Database"' \
  --attribute /domain/endpoint \
  --attribute /domain/port

# Get full component details
si component search 'tag:"production"' --full-component

# Output as JSON
si component search 'schemaName:"S3 Bucket"' --output json
```

---

## Template Execution

Run infrastructure-as-code templates with convergence and idempotency.

### `template run` - Run Template

Run a SI template file.

#### Usage

```bash
si template run <template> --key <invocationKey> [OPTIONS]
```

#### Arguments

| Argument   | Type   | Description           |
| ---------- | ------ | --------------------- |
| `template` | string | Path to template file |

#### Options

| Option                        | Type   | Description                            | Required |
| ----------------------------- | ------ | -------------------------------------- | -------- |
| `-k, --key <invocationKey>`   | string | Invocation key (for idempotency)       | Yes      |
| `-i, --input <file>`          | string | Input data file (JSON or YAML)         | No       |
| `-b, --baseline <file>`       | string | Baseline data file (JSON or YAML)      | No       |
| `-c, --cache-baseline <file>` | string | Cache baseline results to file         | No       |
| `--cache-baseline-only`       | flag   | Exit after writing baseline cache      | No       |
| `--dry-run`                   | flag   | Show planned changes without executing | No       |

#### Environment Variables

- `SI_API_TOKEN` (required)
- `SI_BASE_URL` (required)

#### Examples

```bash
# Run a template
si template run ./templates/infrastructure.ts --key prod-deploy-001

# Run with input data
si template run ./templates/infra.ts \
  --key deploy-002 \
  --input ./inputs/production.yaml

# Run with baseline
si template run ./templates/infra.ts \
  --key deploy-003 \
  --baseline ./baselines/current.yaml

# Preview changes (dry run)
si template run ./templates/infra.ts \
  --key deploy-004 \
  --dry-run

# Cache baseline results
si template run ./templates/infra.ts \
  --key deploy-005 \
  --cache-baseline ./baselines/new-baseline.yaml

# Cache baseline and exit (no execution)
si template run ./templates/infra.ts \
  --key deploy-006 \
  --cache-baseline ./baselines/baseline.yaml \
  --cache-baseline-only
```

---

## Writing Templates

Templates are TypeScript files that export a default function.

### Template Structure

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

### Template Context API

#### `ctx.name(name: string)`

Sets the template name.

```typescript
ctx.name("production-infrastructure");
```

#### `ctx.changeSet(changeSet: string)`

Sets the change set to operate on.

```typescript
ctx.changeSet("production");
```

#### `ctx.search(queries: string[])`

Sets search strings to find components.

```typescript
ctx.search(["schema:AWS EC2 Instance", "tag:production", "tag:web"]);
```

#### `ctx.inputs(schema: ZodSchema)`

Defines input schema with validation and defaults.

```typescript
ctx.inputs(
  z.object({
    environment: z.string().default("production"),
    instanceType: z.string().default("t3.medium"),
    replicas: z.number().min(1).max(10).default(3),
  }),
);
```

#### `ctx.transform(fn: Function)`

Transforms the working set of components.

```typescript
ctx.transform((workingSet) => {
  return workingSet
    .filter((c) => c.name.startsWith("prod-"))
    .map((c) => ({
      ...c,
      tags: [...c.tags, "processed"],
    }));
});
```

#### `ctx.logger`

Logger instance for template output.

```typescript
ctx.logger.info("Starting template execution");
ctx.logger.warn("Warning message");
ctx.logger.error("Error message");
ctx.logger.debug("Debug message");
```

---

## Project Structure

SI projects follow this directory structure:

```
project-root/
├── .siroot                           # Marker file identifying project root
├── .mcp.json                         # MCP server configuration (if using AI agent)
├── .claude/                          # Claude-specific settings (if using Claude)
│   └── settings.local.json
└── schemas/                          # Schema definitions
    └── <schema-name>/
        ├── .format-version
        ├── schema.ts                 # Schema definition
        ├── schema.metadata.json      # Schema metadata
        ├── actions/                  # Action functions
        │   ├── create.ts
        │   ├── create.metadata.json
        │   ├── destroy.ts
        │   ├── destroy.metadata.json
        │   ├── refresh.ts
        │   ├── refresh.metadata.json
        │   ├── update.ts
        │   └── update.metadata.json
        ├── codeGenerators/           # Code generator functions
        │   ├── <codegen-name>.ts
        │   └── <codegen-name>.metadata.json
        ├── management/               # Management functions
        │   ├── <management-name>.ts
        │   └── <management-name>.metadata.json
        └── qualifications/           # Qualification functions
            ├── <qualification-name>.ts
            └── <qualification-name>.metadata.json
```

---

## Common Workflows

:::tip Remember: after running `si schema init`, you can run `si` commands from
**anywhere** inside your project. The CLI automatically finds your project root!
:::

### Setting Up a New Project

```bash
# 1. Initialize project (creates .siroot)
cd /path/to/project
si schema init

# 2. Set up AI agent
si ai-agent init

# 3. Create your first schema
si schema generate scaffold "AWS::EC2::Instance"

# 4. Now you can work from any subdirectory!
cd schemas/AWS::EC2::Instance
si schema generate action "AWS::EC2::Instance" create  # ← Works from here!

# 5. Start coding with AI
si ai-agent start
```

### Working with Existing Schemas

```bash
# 1. Pull schemas from workspace
si schema pull "*"

# 2. Make changes locally
# ... edit files ...

# 3. Push changes back
si schema push --skip-confirmation
```

### Managing Components

```bash
# 1. Search for components
si component search 'schemaName:"AWS EC2 Instance"'

# 2. Get component details
si component get my-server --cache server.yaml

# 3. Modify component
vim server.yaml

# 4. Update component
si component update server.yaml --change-set dev
```

### Running Templates

```bash
# 1. Write template
cat > infrastructure.ts << 'EOF'
import { TemplateContext } from "../src/template.ts";

export default async function (ctx: TemplateContext) {
  ctx.name("my-infrastructure");
  ctx.changeSet("production");
  ctx.search(["schema:AWS EC2 Instance"]);
}
EOF

# 2. Run template (dry run first)
si template run infrastructure.ts \
  --key deploy-001 \
  --dry-run

# 3. Run template for real
si template run infrastructure.ts --key deploy-001
```

---

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

3. Ensure you're using the correct API endpoint:
   ```bash
   si --api-base-url https://api.systeminit.com whoami
   ```

### Project Root Not Found

If the CLI reports it cannot find your project root (no `.siroot` file found),
you have a few options:

1. **Make sure you're in the right place:**

   ```bash
   # Check if you're in or under a project directory
   pwd
   ls -la .siroot  # Should exist somewhere in your project
   ```

2. **Initialize a new project if needed:**

   ```bash
   # Create .siroot in the current directory
   si schema init
   ```

3. **Run from a different directory:**

   ```bash
   # The CLI searches upward, so try from your actual project root
   cd /path/to/project/root
   si schema generate scaffold "AWS::EC2::Instance"
   ```

4. **Explicitly specify the root (rare):**
   ```bash
   # If you must run from outside the project
   si --root /path/to/project schema generate scaffold "AWS::EC2::Instance"
   ```

:::tip The CLI searches **upward** from your current directory for `.siroot`.
This means:

- ✅ Works: `/project/schemas/AWS::EC2::Instance/` → finds `/project/.siroot`
- ✅ Works: `/project/` → finds `/project/.siroot`
- ❌ Fails: `/some/other/directory/` → no `.siroot` found in parent directories
  :::

### Verbose Logging

Enable verbose logging to debug issues:

```bash
# Maximum verbosity (trace level)
si -v4 schema generate scaffold "AWS::EC2::Instance"

# Or specify numeric level (0-4)
si --verbose 4 template run template.ts --key test
```

### AI Agent Issues

If the AI agent isn't working:

1. Check configuration:

   ```bash
   si ai-agent config --show
   ```

2. Verify `.mcp.json` exists:

   ```bash
   cat .mcp.json
   ```

3. Check MCP server directly:

   ```bash
   si ai-agent stdio
   ```

4. Update token if needed:
   ```bash
   si ai-agent config --update-token
   ```
