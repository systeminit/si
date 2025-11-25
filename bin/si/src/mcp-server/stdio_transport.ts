import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { logger } from "./logger.ts";

function deferred<T>() {
  let resolve!: (v: T | PromiseLike<T>) => void;
  let reject!: (reason?: unknown) => void;
  const p = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return Object.assign(p, { resolve, reject });
}

export async function start_stdio(server: McpServer): Promise<void> {
  const transport = new StdioServerTransport();

  const closed = deferred<void>();
  const realClose = transport.close.bind(transport);

  transport.close = async () => {
    try {
      await realClose();
    } finally {
      closed.resolve();
    }
  };

  try {
    await server.connect(transport);
    logger.info("System Initiative MCP server started");

    await closed;
  } catch (error) {
    logger.error(`Error starting mcp server: ${error}`);
    Deno.exit(2);
  }
}
