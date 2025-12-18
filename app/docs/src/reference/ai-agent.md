# AI Agent

The AI Agent is one of the primary ways you will interact with System
Initiative. It allows you to express your intent in natural language, and let
the AI Agent take care of the implementation details.

## Architecture

System Initiative ships a preconfigured version of Claude Code. It integrates
with our
[MCP Server](https://github.com/systeminit/si/tree/main/bin/si/src/ai-agent/mcp-server)
and it also contains a small amount of
[context](https://github.com/systeminit/si/blob/main/bin/si/data/templates/SI_Agent_Context.md.tmpl)
to help the agent perform well.

```mermaid
graph LR
    A[User] --> B[Claude Code]
    B --> C[MCP Server]
    C --> D[System Initiative]
```

No AI Agent is deterministic - ask it to perform the same task twice, and it is
likely to perform it slightly differently each time. That's okay! System
Initiative provides the deterministic layer to manage your infrastructure, and
many tools to ensure you get precisely what you want out of the system.

## Cost Model

System Initiative does not sit between you and the underlying model/agent
provider. We do not mark-up your token usage. It is up to you how you pay for
the underlying model.

## Installing the Agent

For detailed instructions, follow the
[instructions for installing our CLI](../tutorials/install-the-cli).

## Authentication to System Initiative

To authenticate to your System Initiative workspace, run:

```bash
si login
```

This will open our login page and authenticate your CLI. It will them prompt you to choose a workspace:

```bash
si login
✨ info    si              Starting local oauth callback webserver...

Listening on http://[::1]:9003/
Opening login page. If a browser does not open, open https://auth-api.systeminit.com/auth/login?cli_redir=9003 in a browser running on this machine

✨ info    si              Logged in as adam@systeminit.com. Setting as default user.
? Set current workspace 🔎
  GitHub Action CI Workspace - 01JJFCM02Y2Y0FDFH4Y3BTX6SQ - (https://api.systeminit.com)
  Technical Operations - 01J7PP420PHW97TJJB5BA0SQ9A - (https://api.systeminit.com)
  MissionControl Workspace - 01K4TW5QBT0KSAZ2T41YYD5W8M - (https://api.systeminit.com)
```

Select a workspace from the list:

```
? Set current workspace › MissionControl Workspace - 01K4TW5QBT0KSAZ2T41YYD5W8M - (https://api.systeminit.com)
✨ info    si              Workspace token expires at: Fri Jan 09 2026 23:24:33 GMT+0000 (Greenwich Mean Time)
✨ info    si              Set default workspace to: MissionControl Workspace
```

You are now authenticated against this workspace and a workspace token has been generated.

## Starting the Agent

To start the agent, firstly, run:

```bash
si ai-agent init
```

This will use the previously workspace token and generate the correct `CLAUDE.md`,
`.mcp.json` and `.claude/settings.local.json` files that are necessary for
running the agent. The command will also tell you how to install Claude Code.

When the agent is configured, run:

```bash
si ai-agent start
```

This will start Claude Code and connect it to the locally running MCP server.

:::tip

After you switch workspaces e.g. `si workspace switch`, you will want to re-run the AI
agent initialization command to populate the agent with the new workspace API
token.

```bash
si ai-agent init
```

:::

## Prompting the AI Agent

To prompt the AI Agent, you will write your request in plain language. The agent
will then map your request to a series of commands to System Initiative. When
you want a specific outcome, be as specific as possible.

## Validating the Agent is Connected to System Initiative

Validate your credentials to ensure your AI Agent is connected to System
Initiative.

```prompt [Validate Connection to System Initiative]
> Validate my connection to System Initiative
● ✅ Connection validated successfully!

  - User: adam@systeminit.com
  - Workspace ID: 01K6GMC6X7WF066E45FYM7YY8G
  - Role: automation
  - Token expires: October 1, 2026

  Your credentials are working correctly and you have access
  to your System Initiative workspace.
```

## Effective Agent Use

### Be specific when it matters

If you want to be sure something happens, be specific. For example, if you ask
the AI Agent to build a 'server in AWS' for you, it will make its best guess as
to fields like InstanceType or ImageId. These will almost certainly not be the
values you want.

#### Bad Prompt

```prompt [Bad Prompt]
> Make a server in aws
```

#### Good Prompt

```prompt [Good Prompt]
> Make a server in AWS in us-east-1, with 1gb of memory, using ami-12345
```

When you want a specific outcome, be specific in your prompt.

:::tip 

The agent itself is quite effective at improving your prompt. Try asking
it to improve your prompt before you run it!

:::

### Embrace Iteration

System Initiative makes it easy to iterate on your infrastructure.
[Change Sets](../explanation/architecture/change-control.md) ensure that nothing
will change without your approval. Rather than trying to get everything done in
one shot, work in iterations. For example, it's better to:

- Ask the agent to create a best practices network layer
- Configure a cluster
- Deploy an application to the cluster

Than to try and have a single prompt that does all 3. It's easier to review the
work of the agent, and it is more likely to generate a good outcome.

### Clear your Context

The AI Agent has a context window that is quite large, and we try and provide
tools that allow the agent to explore the data it needs from your infrastructure
in a token-friendly way. That said, the longer the conversation chain in the
agent, the more likely it is to start making mistakes.

When you finish a task, we recommend clearing your context and starting fresh on
the next task. Let the LLM explore the information it needs, rather than relying
on having it pick it out of a very long context window.

### Repeatability and Policy

Do not come from the AI Agent. For example, if you want to lay down the exact
same architecture every time (with small variations), you should be using a
template and allowing the Agent to drive it. The same is true with policy -
write qualifications and validations to ensure that components are correct,
rather than relying on careful prompting or context to get it right.

### Review the Agents Work

When working with the AI Agent, review its work. Use the 'R' hot-key to bring up
a review screen whilst on the web app, and ensure that all the correct
attributes and actions are enqueued. While the Agent frequently gets everything
right, it won't always - and that can matter when working with infrastructure.

:::tip 

This is why our pre-configured AI Agent does not allow the agent to apply
or force-apply a change set without your permission. We recommend you keep this
setting!

:::

### More Tips

For more tips on working with the AI Agent:

- [The Claude Code Documentation](https://docs.claude.com/en/docs/claude-code/overview)
- [Prompt Engineering Overview](https://docs.claude.com/en/docs/build-with-claude/prompt-engineering/overview)
- [Claude 4 Best Practices](https://docs.claude.com/en/docs/build-with-claude/prompt-engineering/overview)

## Using other Agents and Models

You can use the System Initiative MCP Server with your own agents and underlying
models. We currently support [OpenAI Codex](https://openai.com/codex/) and
[OpenCode.ai](https://opencode.ai/). To use either of these tools, you can pass
a `--tool` command when configuring and starting the agent:

```bash
si ai-agent init --tool codex
si ai-agent start --tool codex
```

The configuration command will ensure that our AI Agnet is setup appropriately
for the tool.

:::warning

Using other Agents or Models may result in unexpected behavior due to
differences in how they support different features of the
[MCP Protocol](https://modelcontextprotocol.io/docs/getting-started/intro) and
how effectively they utilize tools.

We are happy to work with you to make our MCP Server work well with your agent
and model of choice, but only officially support our AI Agent and the tools it
ships with.

:::

### Running the MCP Server Manually

To run the MCP server directly:

```bash
export SI_API_TOKEN=DEADB33F
si ai-agent stdio
```

First you need to set the `SI_API_TOKEN` environment variable, which is a valid
[workspace API token](./workspaces.md#api-token-management). Then you can run
the MCP server directly.

### Cursor

Cursor uses [mcp.json](https://cursor.com/docs/context/mcp#using-mcpjson) to
configure external MCP Servers. Here is a sample configuration:

```json
{
  "mcpServers": {
    "system-initiative": {
      "type": "stdio",
      "command": "si",
      "args": ["ai-agent", "stdio"],
      "env": {
        "SI_API_TOKEN": "${env:SI_API_TOKEN}"
      }
    }
  }
}
```

:::tip 

Make sure your `SI_API_TOKEN` is set before you launch cursor, so that
the MCP server can authenticate with System Initiative.

:::

### VS Code

Add a `.vscode/mcp.json` file to your workspace, following
[these instructions](https://code.visualstudio.com/docs/copilot/customization/mcp-servers#_configuration-format).

```json
{
  "servers": {
    "systemInitiative": {
      "type": "stdio",
      "command": "si",
      "args": ["ai-agent", "stdio"],
      "env": {
        "SI_API_TOKEN": "${input:SI_API_TOKEN}"
      }
    }
  },
  "inputs": [
    {
      "type": "promptString",
      "id": "SI_API_TOKEN",
      "description": "Your Workspace API Token",
      "password": true
    }
  ]
}
```

VS Code will prompt you to paste your API Token in, which it will store in it's
own 'secure secret storage'.

:::warning 

This is a good example of an environment that may not work well with
our MCP server. If you experience issues, let us know.

:::
