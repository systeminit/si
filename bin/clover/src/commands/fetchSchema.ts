import { CommandFailed } from "../errors.ts";
import _logger from "../logger.ts";
import { Provider } from "../types.ts";
const logger = _logger.ns("fetchSchema").seal();

export async function fetchSchema(provider: Provider) {
  switch (provider) {
    case "aws":
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
      break;
    case "hetzner":
      const url = "https://docs.hetzner.cloud/cloud.spec.json";
      const resp = await fetch(url);
      if (resp.ok) {
        try {
          await Deno.writeTextFile("./src/provider-schemas/hetzner.json", JSON.stringify(await resp.json(), null, 2));
        } catch (err) {
          throw err;
        }
      } else {
        throw new CommandFailed(`Hetzner unreachable at: ${url}`);
      }
      break;
    default:
      console.log(`Unsupported provider type: ${provider}`);
      Deno.exit();
  }
}
