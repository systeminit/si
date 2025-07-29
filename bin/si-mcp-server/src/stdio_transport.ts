import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { logger } from "./logger.ts";

export async function start_stdio(server: McpServer) {
  const transport = new StdioServerTransport();
  try {
    await server.connect(transport);
    logger.info("System Initiative MCP server started");
  } catch (error) {
    logger.error(`Error starting mcp server: ${error}`);
    Deno.exit(2);
  }
}
