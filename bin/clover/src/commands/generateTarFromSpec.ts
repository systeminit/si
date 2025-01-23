import { CommandFailed } from "../errors.ts";
import _logger from "../logger.ts";
const logger = _logger.ns("generateTarFromSpec").seal();

export async function generateTarFromSpec() {
  const td = new TextDecoder();

  const fullPath = Deno.realPathSync("./si-specs");

  for (const dirEntry of Deno.readDirSync(fullPath)) {
    if (
      dirEntry.name.startsWith(".") ||
      dirEntry.name.indexOf("definition.schema") !== -1
    ) {
      continue;
    }

    const pkg = dirEntry.name.split(".")[0];
    if (!pkg) {
      throw new Error(`Could not parse ${dirEntry.name}`);
    }

    console.log(pkg);

    const child = await new Deno.Command(
      "cargo",
      {
        args: [
          "run",
          "--example",
          "si-pkg-json-to-tar",
          `si-specs/${pkg}.json`,
          `tar-packages/${pkg}.tar`,
        ],
      },
    ).output();

    const stdout = td.decode(child.stdout).trim();
    logger.debug(stdout);
    const stderr = td.decode(child.stderr).trim();
    logger.debug(stderr);

    if (!child.success) {
      throw new CommandFailed("failed to generate tar");
    }
  }
}
