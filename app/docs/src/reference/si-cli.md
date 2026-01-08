---
outline: [2, 3, 4]
---

# SI CLI Reference

The `si` CLI is a unified command-line tool for managing System Initiative schemas, templates, and components.

## Global Options

All commands support these global options:

| Option                 | Type   | Description                                    | Default                              |
| ---------------------- | ------ | ---------------------------------------------- | ------------------------------------ |
| `--auth-api-url <URL>` | string | Auth API endpoint URL                          | `https://auth-api.systeminit.com`    |
| `--base-url <URL>`     | string | API endpoint URL                               | -                                    |
| `--api-token <TOKEN>`  | string | Your System Initiative API token               | -                                    |
| `--root <PATH>`        | string | Project root directory (searches for .siroot) | Auto-discovers from current directory |
| `-v, --verbose [level]`| number | Enable verbose logging (0-4)                   | `2`                                  |
| `--no-color`           | flag   | Disable colored output                         | -                                    |
| `-h, --help`           | flag   | Show help                                      | -                                    |
| `-V, --version`        | flag   | Show version                                   | -                                    |

### Verbosity Levels

- `0` - Errors only
- `1` - Errors + Warnings
- `2` - Errors + Warnings + Info (default)
- `3` - Errors + Warnings + Info + Debug
- `4` - Errors + Warnings + Info + Debug + Trace

## Environment Variables

| Variable           | Description                                                       |
| ------------------ | ----------------------------------------------------------------- |
| `SI_AUTH_API_URL`  | Auth API endpoint URL                                             |
| `SI_BASE_URL`      | API endpoint URL                                                  |
| `SI_API_TOKEN`     | Your System Initiative Workspace API token (required for authenticated commands) |
| `SI_ROOT`          | Project root directory (searches for .siroot if not specified)   |

---

## whoami

Displays authenticated user information.

> Syntax

```bash
si whoami
```

### Parameters

None


## completion

Generate shell completions.

### completion bash

Generate shell completions for bash.

> Syntax

```bash
si completion bash
```

#### Parameters

None

---

### completion fish

Generate shell completions for fish.

> Syntax

```bash
si completion fish
```

#### Parameters

None

---

### completion zsh

Generate shell completions for zsh.

> Syntax

```bash
si completion zsh
```

#### Parameters

None

---

## login

Login to System Initiative.

> Syntax

```bash
si login
```

### Parameters

None


## logout

Logout from System Initiative.

> Syntax

```bash
si logout [OPTIONS]
```

### Parameters

| Name     | Type | Required | Description                                           | Default |
| -------- | ---- | -------- | ----------------------------------------------------- | ------- |
| --clear  | flag | false    | Also delete stored tokens for the current user from disk | false |


## workspace

Manage workspaces you have access to.

### workspace switch

Switch to a different workspace.

> Syntax

```bash
si workspace switch [workspace]
```

#### Parameters

| Name      | Type   | Required | Description             |
| --------- | ------ | -------- | ----------------------- |
| workspace | string | false    | Workspace to switch to  |


### workspace create

Create a new workspace.

> Syntax

```bash
si workspace create <name>
```

#### Parameters

| Name | Type   | Required | Description         |
| ---- | ------ | -------- | ------------------- |
| name | string | true     | Name of workspace   |


### workspace leave

Leave a workspace (remove yourself as a member).

> Syntax

```bash
si workspace leave <workspace>
```

#### Parameters

| Name      | Type   | Required | Description                                     |
| --------- | ------ | -------- | ----------------------------------------------- |
| workspace | string | true     | Workspace ID or name to leave                   |

::: warning
You cannot leave your current workspace. Switch to a different workspace first using `si workspace switch`.
:::


### workspace delete

Delete a workspace (soft delete - can be recovered by contacting support).

> Syntax

```bash
si workspace delete <workspace>
```

#### Parameters

| Name      | Type   | Required | Description                                     |
| --------- | ------ | -------- | ----------------------------------------------- |
| workspace | string | true     | Workspace ID or name to delete                  |

::: warning
You cannot delete your current workspace. Switch to a different workspace first using `si workspace switch`.

To recover a deleted workspace, contact customer service at support@systeminit.com. Note that this operation will leave any existing resources running.
:::

---

### workspace members

Manage workspace members (view and invite/update).

#### workspace members list

List all members of the current workspace.

> Syntax

```bash
si workspace members list
```

##### Parameters

None

##### Output

Displays a table showing:
- Email address
- Nickname
- Role (OWNER, APPROVER, or COLLABORATOR)

Members are sorted by role (Owner first, then Approvers, then Collaborators).

##### Example

```bash
$ si workspace members list

Members of workspace "Production":

Email                    Nickname  Role
alice@example.com        Alice     OWNER
bob@example.com          Bob       APPROVER
charlie@example.com      Charlie   COLLABORATOR

Total members: 3
```

---

#### workspace members manage

Invite new members or update existing member roles in the current workspace.

> Syntax

```bash
si workspace members manage [email] [OPTIONS]
```

##### Parameters

| Name         | Type   | Required | Description                                                     |
| ------------ | ------ | -------- | --------------------------------------------------------------- |
| email        | string | false    | Single email to invite as collaborator                          |
| --approvers  | string | false    | Comma-separated list of emails to invite/update as approvers    |

##### Behavior

**For new members:**
- Members are invited to the workspace
- Default role is collaborator
- Members invited with `--approvers` are promoted to approver role after invitation

**For existing members:**
- Detects if the user is already a member
- Updates their role if different from the requested role
- Supports both role upgrades (collaborator → approver) and downgrades (approver → collaborator)
- Skips invitation if user already has the correct role

##### Examples

```bash
# Invite a single collaborator
si workspace members manage alice@example.com

# Invite multiple collaborators
si workspace members manage alice@example.com,bob@example.com

# Invite/promote users to approvers
si workspace members manage --approvers charlie@example.com,dave@example.com

# Promote existing collaborator to approver
si workspace members manage --approvers alice@example.com

# Demote existing approver to collaborator
si workspace members manage bob@example.com
```

::: tip
All members are invited as collaborators by default. Use `--approvers` to invite or promote members to the approver role, which allows them to approve change sets and invite other members.
:::

---

## change-set

Manage change sets.

### change-set create

Create a new change set.

> Syntax

```bash
si change-set create <name>
```

#### Parameters

| Name | Type   | Required | Description          |
| ---- | ------ | -------- | -------------------- |
| name | string | true     | Name of change set   |


### change-set abandon

Abandon (delete) a change set.

> Syntax

```bash
si change-set abandon <change-set-id-or-name>
```

#### Parameters

| Name                  | Type   | Required | Description               |
| --------------------- | ------ | -------- | ------------------------- |
| change-set-id-or-name | string | true     | Change set ID or name     |


### change-set open

Open a change set in the browser.

> Syntax

```bash
si change-set open <change-set-id-or-name>
```

#### Parameters

| Name                  | Type   | Required | Description               |
| --------------------- | ------ | -------- | ------------------------- |
| change-set-id-or-name | string | true     | Change set ID or name     |


### change-set apply

Apply a change set to HEAD.

> Syntax

```bash
si change-set apply <change-set-id-or-name>
```

#### Parameters

| Name                  | Type   | Required | Description               |
| --------------------- | ------ | -------- | ------------------------- |
| change-set-id-or-name | string | true     | Change set ID or name     |


### change-set list

List all change sets.

> Syntax

```bash
si change-set list
```

#### Parameters

None


## ai-agent

Manages the SI AI Agent (MCP server).

### ai-agent init

Initialize AI agent (one-time setup: configure token and create MCP files).

> Syntax

```bash
si ai-agent init [OPTIONS]
```

#### Parameters

| Name         | Type   | Required | Description                                              | Default           |
| ------------ | ------ | -------- | -------------------------------------------------------- | ----------------- |
| --target-dir | string | false    | Directory to create config files                         | Current directory |
| --tool       | string | false    | AI tool to use: `claude`, `codex`, `opencode`, `cursor`  | -                 |


### ai-agent start

Launch Claude Code (MCP server starts automatically).

> Syntax

```bash
si ai-agent start [OPTIONS]
```

#### Parameters

| Name   | Type   | Required | Description                      | Default  |
| ------ | ------ | -------- | -------------------------------- | -------- |
| --tool | string | false    | AI tool to launch                | `claude` |


### ai-agent config

View or update AI agent configuration.

> Syntax

```bash
si ai-agent config [OPTIONS]
```

#### Parameters

| Name           | Type   | Required | Description                                              | Default |
| -------------- | ------ | -------- | -------------------------------------------------------- | ------- |
| --show         | flag   | false    | Show current configuration (default if no other options) | false   |
| --update-token | flag   | false    | Update the API token                                     | false   |
| --tool         | string | false    | Update the AI tool: `claude`, `cursor`, `windsurf`, `none` | -    |


### ai-agent stdio

Run MCP server in stdio mode (for external AI tools to connect).

> Syntax

```bash
si ai-agent stdio
```

#### Parameters

None


## schema

Manage schemas: initialize project, generate functions locally, pull from and push to remote workspaces.

### schema init

Initialize a new System Initiative project in the current or specified directory.

> Syntax

```bash
si schema init [ROOT_PATH]
```

#### Parameters

| Name      | Type   | Required | Description             | Default           |
| --------- | ------ | -------- | ----------------------- | ----------------- |
| ROOT_PATH | string | false    | Directory to initialize | Current directory |


### schema action generate

Generate action functions for schemas.

> Syntax

```bash
si schema action generate [SCHEMA_NAME] [ACTION_NAME]
```

#### Parameters

| Name        | Type   | Required | Description                                            |
| ----------- | ------ | -------- | ------------------------------------------------------ |
| SCHEMA_NAME | string | false    | Name of the schema                                     |
| ACTION_NAME | string | false    | Action name: `create`, `destroy`, `refresh`, `update`  |


### schema authentication generate

Generate authentication functions for schemas.

> Syntax

```bash
si schema authentication generate [SCHEMA_NAME] [AUTH_NAME]
```

#### Parameters

| Name        | Type   | Required | Description                         |
| ----------- | ------ | -------- | ----------------------------------- |
| SCHEMA_NAME | string | false    | Name of the schema                  |
| AUTH_NAME   | string | false    | Name of the authentication function |


### schema codegen generate

Generate code generator functions for schemas.

> Syntax

```bash
si schema codegen generate [SCHEMA_NAME] [CODEGEN_NAME]
```

#### Parameters

| Name         | Type   | Required | Description                |
| ------------ | ------ | -------- | -------------------------- |
| SCHEMA_NAME  | string | false    | Name of the schema         |
| CODEGEN_NAME | string | false    | Name of the code generator |


### schema management generate

Generate management functions for schemas.

> Syntax

```bash
si schema management generate [SCHEMA_NAME] [MANAGEMENT_NAME]
```

#### Parameters

| Name            | Type   | Required | Description                     |
| --------------- | ------ | -------- | ------------------------------- |
| SCHEMA_NAME     | string | false    | Name of the schema              |
| MANAGEMENT_NAME | string | false    | Name of the management function |


### schema qualification generate

Generate qualification functions for schemas.

> Syntax

```bash
si schema qualification generate [SCHEMA_NAME] [QUALIFICATION_NAME]
```

#### Parameters

| Name               | Type   | Required | Description                        |
| ------------------ | ------ | -------- | ---------------------------------- |
| SCHEMA_NAME        | string | false    | Name of the schema                 |
| QUALIFICATION_NAME | string | false    | Name of the qualification function |


### schema scaffold generate

Generate a complete schema scaffold with all default functions.

> Syntax

```bash
si schema scaffold generate [SCHEMA_NAME]
```

#### Parameters

| Name        | Type   | Required | Description                    |
| ----------- | ------ | -------- | ------------------------------ |
| SCHEMA_NAME | string | false    | Name of the schema to scaffold |


### schema pull

Pulls schemas from your remote System Initiative workspace. Supports wildcard patterns like `Fastly::*` to pull all schemas in a category, or `*` to pull all schemas.

> Syntax

```bash
si schema pull [SCHEMA_NAME...]
```

#### Parameters

| Name        | Type     | Required | Description                               |
| ----------- | -------- | -------- | ----------------------------------------- |
| SCHEMA_NAME | string[] | false    | Schema names to pull (supports wildcards) |
| --builtins  | flag     | false    | Include builtin schemas (schemas you don't own) |


### schema push

Pushes schemas to your remote System Initiative workspace.

> Syntax

```bash
si schema push [SCHEMA_NAME...]
```

#### Parameters

| Name                    | Type     | Required | Description                                        |
| ----------------------- | -------- | -------- | -------------------------------------------------- |
| SCHEMA_NAME             | string[] | false    | Schema names to push                               |
| -s, --skip-confirmation | flag     | false    | Skip confirmation prompt                           |
| -b, --update-builtins   | flag     | false    | Change builtin schema, without creating overlays   |

---

### schema contribute

Contribute a schema to the module index (works on HEAD change set only).

> Syntax

```bash
si schema contribute <SCHEMA>
```

#### Parameters

| Name   | Type   | Required | Description         |
| ------ | ------ | -------- | ------------------- |
| SCHEMA | string | true     | Name of the schema  |

---

### schema overlay

Manage schema overlays: generate overlay functions and push them to remote workspaces.

#### schema overlay action generate

Generate action overlay functions.

> Syntax

```bash
si schema overlay action generate [SCHEMA_NAME] [OVERLAY_NAME]
```

##### Parameters

| Name         | Type   | Required | Description                   |
| ------------ | ------ | -------- | ----------------------------- |
| SCHEMA_NAME  | string | false    | Name of the schema            |
| OVERLAY_NAME | string | false    | Name of the action overlay    |


#### schema overlay authentication generate

Generate authentication overlay functions.

> Syntax

```bash
si schema overlay authentication generate [SCHEMA_NAME] [OVERLAY_NAME]
```

##### Parameters

| Name         | Type   | Required | Description                          |
| ------------ | ------ | -------- | ------------------------------------ |
| SCHEMA_NAME  | string | false    | Name of the schema                   |
| OVERLAY_NAME | string | false    | Name of the authentication overlay   |


#### schema overlay codegen generate

Generate code generator overlay functions.

> Syntax

```bash
si schema overlay codegen generate [SCHEMA_NAME] [OVERLAY_NAME]
```

##### Parameters

| Name         | Type   | Required | Description                   |
| ------------ | ------ | -------- | ----------------------------- |
| SCHEMA_NAME  | string | false    | Name of the schema            |
| OVERLAY_NAME | string | false    | Name of the codegen overlay   |


#### schema overlay management generate

Generate management overlay functions.

> Syntax

```bash
si schema overlay management generate [SCHEMA_NAME] [OVERLAY_NAME]
```

##### Parameters

| Name         | Type   | Required | Description                     |
| ------------ | ------ | -------- | ------------------------------- |
| SCHEMA_NAME  | string | false    | Name of the schema              |
| OVERLAY_NAME | string | false    | Name of the management overlay  |


#### schema overlay qualification generate

Generate qualification overlay functions.

> Syntax

```bash
si schema overlay qualification generate [SCHEMA_NAME] [OVERLAY_NAME]
```

##### Parameters

| Name         | Type   | Required | Description                        |
| ------------ | ------ | -------- | ---------------------------------- |
| SCHEMA_NAME  | string | false    | Name of the schema                 |
| OVERLAY_NAME | string | false    | Name of the qualification overlay  |


#### schema overlay push

Pushes overlay funcs to your remote System Initiative workspace.

> Syntax

```bash
si schema overlay push [SCHEMA_NAME...]
```

##### Parameters

| Name        | Type     | Required | Description                     |
| ----------- | -------- | -------- | ------------------------------- |
| SCHEMA_NAME | string[] | false    | Schema names (overlays) to push |


## component

Component-related operations.

### component get

Get component data by name or ID.

> Syntax

```bash
si component get <component> [OPTIONS]
```

#### Parameters

| Name             | Type   | Required | Description                                      | Default |
| ---------------- | ------ | -------- | ------------------------------------------------ | ------- |
| component        | string | true     | Component name or ID                             | -       |
| -c, --change-set | string | false    | Change set ID or name                            | `HEAD`  |
| -o, --output     | string | false    | Output format: `info`, `json`, or `yaml`         | `info`  |
| --cache          | string | false    | Cache output to file; format determined by file extension (.json, .yaml, .yml) | - |
| --raw            | flag   | false    | Output raw API response as JSON and exit         | false   |


### component create

Create component from JSON/YAML file (idempotent).

> Syntax

```bash
si component create <input-file> [OPTIONS]
```

#### Parameters

| Name             | Type   | Required | Description                                      | Default |
| ---------------- | ------ | -------- | ------------------------------------------------ | ------- |
| input-file       | string | true     | Path to input file (JSON or YAML)               | -       |
| -c, --change-set | string | false    | Change set ID or name                            | `HEAD`  |
| -o, --output     | string | false    | Output format: `info`, `json`, or `yaml`         | `info`  |
| --cache          | string | false    | Cache output to file; format determined by file extension | - |
| --raw            | flag   | false    | Output raw API response as JSON and exit         | false   |


### component update

Update a component from JSON/YAML file (idempotent).

> Syntax

```bash
si component update <input-file> --change-set <id-or-name> [OPTIONS]
```

#### Parameters

| Name             | Type   | Required | Description                                           |
| ---------------- | ------ | -------- | ----------------------------------------------------- |
| input-file       | string | true     | Path to input file (JSON or YAML)                     |
| -c, --change-set | string | true     | Change set ID or name                                 |
| --component      | string | false    | Component ID or name (overrides componentId from file)|
| --dry-run        | flag   | false    | Show diff without applying changes                    |


### component delete

Delete a component by name or ID.

> Syntax

```bash
si component delete <component> --change-set <id-or-name> [OPTIONS]
```

#### Parameters

| Name             | Type   | Required | Description                          |
| ---------------- | ------ | -------- | ------------------------------------ |
| component        | string | true     | Component name or ID                 |
| -c, --change-set | string | true     | Change set ID or name                |
| --dry-run        | flag   | false    | Preview deletion without applying changes |


### component erase

Erase a component by name or ID.

> Syntax

```bash
si component erase <component> --change-set <id-or-name> [OPTIONS]
```

#### Parameters

| Name             | Type   | Required | Description                               |
| ---------------- | ------ | -------- | ----------------------------------------- |
| component        | string | true     | Component name or ID                      |
| -c, --change-set | string | true     | Change set ID or name                     |
| --dry-run        | flag   | false    | Preview deletion without applying changes |


### component search

Search for components using a search query.

> Syntax

```bash
si component search <query> [OPTIONS]
```

#### Parameters

| Name              | Type     | Required | Description                                           | Default |
| ----------------- | -------- | -------- | ----------------------------------------------------- | ------- |
| query             | string   | true     | Search query                                          | -       |
| -c, --change-set  | string   | false    | Change set ID or name                                 | `HEAD`  |
| -o, --output      | string   | false    | Output format: `info`, `json`, or `yaml`              | `info`  |
| -a, --attribute   | string[] | false    | Attribute paths to include in output (repeatable)     | -       |
| --full-component  | flag     | false    | Show full component details for each result           | false   |


### component upgrade

Upgrade component(s) to the latest schema version.

This command checks if components can be upgraded before creating a change set, preventing orphaned change sets for no-op operations. If no change set is specified, it automatically creates one with a descriptive name.

> Syntax

```bash
# Upgrade a specific component
si component upgrade <component> [OPTIONS]

# Upgrade all upgradable components
si component upgrade --all [OPTIONS]
```

#### Parameters

| Name               | Type   | Required | Description                                                      | Default |
| ------------------ | ------ | -------- | ---------------------------------------------------------------- | ------- |
| component          | string | false    | Component name or ID (required unless --all is specified)        | -       |
| -c, --change-set   | string | false    | Change set ID or name (creates new change set if not specified)  | -       |
| --all              | flag   | false    | Upgrade all upgradable components (required if no component specified) | false |
| --schema-category  | string | false    | Filter by schema category (e.g., `AWS::EC2`) when using --all   | -       |
| --dry-run          | flag   | false    | Preview upgrades without applying changes                        | false   |

#### Examples

```bash
# Upgrade a specific component (auto-creates change set)
si component upgrade my-ec2-instance

# Upgrade a component in an existing change set
si component upgrade my-s3-bucket -c my-changes

# Upgrade all upgradable components
si component upgrade --all

# Upgrade only AWS::EC2 components
si component upgrade --all --schema-category AWS::EC2

# Preview what would be upgraded
si component upgrade --all --dry-run
```

#### Behavior

- Checks if components can be upgraded in HEAD **before** creating a change set
- If nothing can be upgraded, exits cleanly without creating a change set
- For bulk upgrades (`--all`), processes components one at a time
- Individual component failures don't stop bulk upgrades
- Auto-created change sets are abandoned on error
- Returns exit code 1 if any components fail to upgrade

#### Notes

- Components can only be upgraded in a change set, not on HEAD
- The `canBeUpgraded` flag indicates if a newer schema version is available
- Use `--schema-category` to filter bulk upgrades (e.g., `AWS::EC2`, `Hetzner::Cloud`)
- Schema category filters use the format `Provider::Service` (e.g., `Microsoft.Network` won't work, use `Microsoft`)


## secret

Manage secrets and credentials.

### secret create

Create a new secret.

> Syntax

```bash
si secret create <secret-type> [OPTIONS]
```

#### Parameters

| Name                | Type   | Required | Description                                              | Default |
| ------------------- | ------ | -------- | -------------------------------------------------------- | ------- |
| secret-type         | string | true     | Type of secret to create                                 | -       |
| --name              | string | false    | Name for the secret instance                             | -       |
| --description       | string | false    | Description for the secret                               | -       |
| -c, --change-set    | string | false    | Change set ID or name (creates new change set if not specified) | - |
| --use-local-profile | flag   | false    | Discover credentials from local environment (e.g., AWS credentials) | false |
| --interactive       | flag   | false    | Prompt for all values interactively                      | false   |
| --dry-run           | flag   | false    | Show what would be created without making changes        | false   |


### secret update

Update an existing secret by component name or ID.

> Syntax

```bash
si secret update <component-name-or-id> [OPTIONS]
```

#### Parameters

| Name                | Type   | Required | Description                                              | Default |
| ------------------- | ------ | -------- | -------------------------------------------------------- | ------- |
| component-name-or-id| string | true     | Component name or ID                                     | -       |
| --name              | string | false    | New name for the secret                                  | -       |
| --description       | string | false    | New description for the secret                           | -       |
| -c, --change-set    | string | false    | Change set ID or name (creates new change set if not specified) | - |
| --use-local-profile | flag   | false    | Discover credentials from local environment (e.g., AWS credentials) | false |
| --interactive       | flag   | false    | Prompt for all values interactively                      | false   |
| --dry-run           | flag   | false    | Show what would be updated without making changes        | false   |


## template

Manages System Initiative templates.

### template generate

Generate a new template structure file.

> Syntax

```bash
si template generate <name> [OPTIONS]
```

#### Parameters

| Name            | Type   | Required | Description                                      | Default           |
| --------------- | ------ | -------- | ------------------------------------------------ | ----------------- |
| name            | string | true     | Name of the template                             | -                 |
| -o, --output-dir| string | false    | Output directory for the template file           | Current directory |


### template run

Run a SI template file (local path or remote URL).

> Syntax

```bash
si template run <template> --key <invocationKey> [OPTIONS]
```

#### Parameters

| Name                  | Type   | Required | Description                                           |
| --------------------- | ------ | -------- | ----------------------------------------------------- |
| template              | string | true     | Path to template file or remote URL                   |
| -k, --key             | string | true     | Invocation key for the template; used for idempotency |
| -i, --input           | string | false    | Path to input data file (JSON or YAML); validated against template's input schema |
| -b, --baseline        | string | false    | Path to baseline data file (JSON or YAML)            |
| -c, --cache-baseline  | string | false    | Path to cache baseline results; format determined by file extension (.json, .yaml, .yml) |
| --cache-baseline-only | flag   | false    | Exit after writing baseline cache (requires --cache-baseline) |
| --dry-run             | flag   | false    | Show planned changes without executing them           |


## policy

Policy management operations.

### policy evaluate

Evaluate policies against infrastructure components.

This command evaluates one or more compliance policies written in markdown format against your System Initiative infrastructure. The evaluation process uses Claude AI to:
1. Extract policy structure from the markdown document(s)
2. Collect infrastructure data from System Initiative based on queries
3. Evaluate components against the policy requirements
4. Generate detailed markdown reports with findings
5. Upload results to System Initiative (unless `--no-upload` is specified)

All output files are organized in a single folder (timestamped or custom-named) for easy management.

> Syntax

```bash
# Evaluate a single policy file
si policy evaluate <file-path> --name <policy-name> [OPTIONS]

# Evaluate all policies in a directory
si policy evaluate <directory-path> --all [OPTIONS]
```

#### Parameters

| Name                    | Type   | Required | Description                                           | Default   |
| ----------------------- | ------ | -------- | ----------------------------------------------------- | --------- |
| file-path               | string | true     | Path to a policy markdown file or directory           | -         |
| -n, --name              | string | conditional | Name for the policy evaluation (required for single files, derived from filename when using `--all`) | - |
| --all                   | flag   | false    | Evaluate all `.md` files in a directory (only works with directories) | false |
| -c, --change-set        | string | false    | Change set ID or name to evaluate against             | `HEAD`    |
| -o, --output-folder     | string | false    | Folder name to organize results                       | Timestamp (e.g., `2026-01-08T01:10:15Z`) |
| --no-upload             | flag   | false    | Skip uploading the policy evaluation results          | false     |

#### Output Files

The command creates a folder containing:
- `{policy-name}-extracted.json` - Extracted policy structure
- `{policy-name}-source-data.json` - Infrastructure data collected from System Initiative
- `{policy-name}-evaluation.json` - Policy evaluation results
- `report.md` - Human-readable markdown report with findings

#### Examples

```bash
# Evaluate a single policy file
si policy evaluate policies/vpc-compliance.md --name "VPC Compliance Policy"

# Evaluate against a specific change set
si policy evaluate policies/vpc-compliance.md --name "VPC Policy" -c my-changeset

# Organize results in a custom folder
si policy evaluate policies/vpc-compliance.md --name "VPC Policy" -o vpc-audit

# Skip upload stage
si policy evaluate policies/vpc-compliance.md --name "VPC Policy" --no-upload

# Evaluate all policies in a directory
si policy evaluate policies/ --all

# Evaluate all policies with custom options
si policy evaluate policies/ --all -c my-changeset -o audit-results
```

#### Behavior

**Single File Mode:**
- Requires `--name` parameter to identify the policy
- Evaluates one policy file
- Policy name is used in the uploaded report

**Directory Mode (with `--all`):**
- Automatically finds all `.md` files in the directory
- Policy name is derived from filename (e.g., `vpc-compliance.md` → `vpc-compliance`)
- Each policy is evaluated independently
- If one policy fails, others continue processing
- Summary shows results for all evaluated policies

#### Validation

The command validates inputs and provides clear error messages:

```bash
# Directory without --all
$ si policy evaluate policies/
Error: The path "policies/" is a directory. Please use the --all flag to evaluate all policy files in the directory.

# File with --all but no name
$ si policy evaluate policy.md --all
Error: When evaluating a single file, the --name option is required. The --all flag only applies to directories.

# Empty directory
$ si policy evaluate empty/ --all
Error: No policy files (.md) found in directory: empty/

# Non-existent path
$ si policy evaluate invalid/path
Error: Path does not exist: invalid/path
```

#### Policy Markdown Format

Your policy markdown file should include:

1. **Policy Title** - The main heading (`# Policy Title`)
2. **Policy Section** - Contains the policy requirements and exceptions
3. **Source Data Section** - YAML block with queries to collect infrastructure data
4. **Output Tags Section** - YAML block with tags for categorization

Example structure:

```markdown
# All VPCs must use private IP ranges

## Policy

All VPCs must be configured with private IP address ranges (10.0.0.0/8, 172.16.0.0/12, or 192.168.0.0/16).

### Exceptions

Test VPCs may use non-standard ranges for development purposes.

## Source Data

### System Initiative

```yaml
all-vpcs: "schema:\"AWS::EC2::VPC\""
```

## Output Tags

```yaml
tags:
  - networking
  - security
```
```

#### Evaluation Results

The command displays:
- **PASS** (green) - All components comply with the policy
- **FAIL** (yellow warning) - One or more components fail the policy

Exit codes:
- `0` - Evaluation completed successfully (policy may pass or fail)
- `1` - Evaluation process encountered an error

#### Requirements

- `ANTHROPIC_API_KEY` environment variable must be set
- Active System Initiative workspace authentication

::: tip
The policy evaluation uses the Claude AI agent to analyze your infrastructure. The agent has read-only access and will not make any changes to your System Initiative workspace.
:::

