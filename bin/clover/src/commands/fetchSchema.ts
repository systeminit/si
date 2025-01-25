import { CommandFailed } from "../errors.ts";
import _logger from "../logger.ts";
const logger = _logger.ns("fetchSchema").seal();

export async function fetchSchema() {
  const td = new TextDecoder();
  const child = await new Deno.Command(
    Deno.execPath(),
    {
      args: ["run", "updateSchema"],
    },
  ).output();

  const stdout = td.decode(child.stdout).trim();
  logger.debug(stdout);
  const stderr = td.decode(child.stderr).trim();
  logger.debug(stderr);

  if (!child.success) {
    throw new CommandFailed("failed to fetch schema");
  }
}
