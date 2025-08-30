import { Command } from "@cliffy/command";
import { start_stdio } from "./stdio_transport.ts";
import { createServer } from "./server.ts";
import { analytics } from "./analytics.ts";
import { setAiAgentUserFlag } from "./user_state.ts";

export async function run() {
  const command = new Command()
    .name("si-mcp-server")
    .version("0.1.0")
    .description("MCP Server for System Initiative")
    .globalEnv(
      "SI_API_TOKEN=<string>",
      "The System Initiative API Token",
    )
    .globalEnv(
      "SI_WORKSPACE_ID=<string>",
      "The System Initiative Workspace to connect the MCP Server to",
    )
    .globalEnv(
      "SI_BASE_URL=<string>",
      "The base URI for System Initiative. Defaults to https://api.systeminit.com",
    )
    .action(() => {
      command.showHelp();
      Deno.exit(1);
    })
    .command(
      "stdio",
      "Start the SI MCP Server over the stdio transport",
    )
    .action(async () => {
      await analytics.trackServerStart();
      await setAiAgentUserFlag();
      const server = createServer();
      await start_stdio(server);
    });

  await command.parse(Deno.args);
}
