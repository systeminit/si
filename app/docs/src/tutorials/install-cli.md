# Install CLI

System Initiative ships a binary for you to install and configure.

## Download & Install Binary

```shellscript
## Linux / macOS
curl -fsSL https://auth.systeminit.com/install.sh | sh

## Windows
irm https://auth.systeminit.com/install.ps1 | iex
```

## Configure

There are only three environment variables that the configure the binary:

- `SI_API_TOKEN` Your System Initiative workspace API token. This is required.
- `SI_BASE_URL` the URL the CLI will use to find the API endpoints. This
  defaults to `https://api.systeminit.com`, which is the API endpoint for the
  managed SaaS product. If you are running locally, or have deployed System
  Initiative elsewhere, you will need to set this variable.
- `SI_ROOT` the local directory on disk where you want to keep references to
  schema data when working with it via the CLI. This is not required.

Alternatively, if you don't want to use environment variables, the values can be
passed at the time of script execution with these flags:

- `--api-token <TOKEN>` API authentication token
- `--api-base-url <URL>` Override the API endpoint
- `--root <PATH>` Specify root directory

### To get your API token:

1. Visit https://auth.systeminit.com/workspaces
2. Click the gear icon for your workspace
3. Select "API Tokens"
4. Generate a new token (recommended: 1 year expiration)
5. Copy and paste when prompted

## Launch the CLI

There is a one-time setup command to run first if you want to use an AI agent.

```shellscript
$ si ai-agent init
```

This will ask for your API key again to configure the MCP server & tools to use.

To start the AI agent:

```shellscript
$ si ai-agent start
```

To see all the commands available:

```shellscript
$ si --help
```

### Reference Materials

Use the CLI to work with:

- [Change Sets](../reference/change-sets.md)
- [Components](../reference/components.md)
- [Secrets](../reference/secrets.md)
