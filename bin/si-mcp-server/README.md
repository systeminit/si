# System Initiative MCP Server

A Model Context Protocol (MCP) server that provides Claude Code with direct access to System Initiative's API for managing infrastructure components, change sets, schemas, and actions.

## Development Setup

### Environment Variables

Required:

```bash
export SI_API_TOKEN="your-token-here"
export SI_BASE_URL="http://localhost:5380"  # Defaults to production if not overridden
```

Optional analytics configuration:

```bash
export POSTHOG_API_KEY="dev-posthog-key"  # Override if you want to use the Dev Project to test tracking changes
```

### Testing with MCP Inspector

The MCP Inspector provides a web interface for testing tools and debugging:

```bash
# Start the inspector
deno task inspector
```

This opens a browser interface where you can:

- Test individual tools with parameters
- Inspect tool schemas and validate responses  
- Debug authentication and connection issues
- View real-time tool execution

### Testing with Claude Code

Configure Claude Code to use your local server (without running in a container):

```bash
# From the repository root
claude mcp add si-mcp-server-dev -- deno run --allow-env --allow-net bin/si-mcp-server/main.ts stdio

# Or from within si-mcp-server directory  
claude mcp add si-mcp-server-dev -- deno run --allow-env --allow-net main.ts stdio

# Verify it's working
claude mcp list
```

## Available Tasks

| Task | Command | Description |
|------|---------|-------------|
| `dev` | `deno task dev` | Run with auto-reload for development |
| `inspector` | `deno task inspector` | Start MCP Inspector for testing |
| `build` | `deno task build` | Compile to standalone binary |
| `docker:build` | `deno task docker:build` | Build Docker image |
| `docker:run` | `deno task docker:run` | Run Docker container |

## Adding New Tools

1. Create tool file in `src/tools/`
2. Import required dependencies and `withAnalytics` from `commonBehavior.ts`
3. Define Zod schemas for input/output validation
4. Wrap handler with `withAnalytics(toolName, async () => { ... })`
5. Export tool registration function
6. Register in `src/server.ts`
