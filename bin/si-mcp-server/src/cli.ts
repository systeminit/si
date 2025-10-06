import { Command } from "@cliffy/command";
import { start_stdio } from "./stdio_transport.ts";
import { createServer } from "./server.ts";
import { analytics } from "./analytics.ts";
import { setAiAgentUserFlag } from "./user_state.ts";

export async function run() {
  const command = new Command()
    .name("si-mcp-server")
    .version("0.1.0")
    .description("MCP Server for System Initiative5000")
    .globalEnv("SI_API_TOKEN=<string>", "The System Initiative API Token")
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
    .command("stdio", "Start the SI MCP Server over the stdio transport")
    .action(async () => {
      await analytics.trackServerStart();
      await setAiAgentUserFlag();

      const server = createServer();

      let ended = false;
      const shutdown = async (reason: string, exitCode: number | null = 0) => {
        if (ended) return;
        ended = true;
        console.log("END EVENT:", reason);
        try {
          analytics.trackServerEnd();
        } catch (_err) {
          // ignore
        }

        // This is a sleep to let the events flush before we shut down the process
        await new Promise((r) => setTimeout(r, 25));
        if (exitCode !== null) Deno.exit(exitCode);
      };

      const onSigInt = () => {
        shutdown("SIGINT", 0);
      };
      const onSigTerm = () => {
        shutdown("SIGTERM", 0);
      };
      Deno.addSignalListener("SIGINT", onSigInt);
      Deno.addSignalListener("SIGTERM", onSigTerm);

      try {
        await start_stdio(server);
        await shutdown("transport_closed", null);
      } catch (err: unknown) {
        const name = err instanceof Error
          ? err.name
          : ((err as { name?: string })?.name ?? "unknown");
        await shutdown(`uncaught_error:${name}`, 1);
      } finally {
        Deno.removeSignalListener("SIGINT", onSigInt);
        Deno.removeSignalListener("SIGTERM", onSigTerm);
      }
    });

  await command.parse(Deno.args);
}
