/**
 * Standalone si-mcp-server binary entry point
 *
 * This is a thin wrapper that imports from the vendored MCP server code
 * in bin/si/src/mcp-server/ (which is the source of truth).
 *
 * The same code is also bundled into the si CLI binary and available
 * via `si mcp-server stdio`.
 */

import { run } from "../si/src/ai-agent/mcp-server/cli.ts";

if (import.meta.main) {
  await run();
}
