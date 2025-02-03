import _logger from "./logger.ts";
import _ from "npm:lodash";

const logger = _logger.ns("packageGen").seal();
export const EXISTING_PACKAGES = "existing-packages/spec.json";

export async function getExistingSpecs(moduleIndexUrl: string): Promise<Record<string, string>> {
  logger.debug("Getting existing specs...");
  const args = [
    "run",
    "//bin/hoist:hoist",
    "--",
    "--endpoint",
    moduleIndexUrl,
    "write-existing-modules-spec",
    "--out",
    EXISTING_PACKAGES,
  ];
  logger.info(`Running: buck2 ${args.join(" ")}`);
  const child = new Deno.Command(
    "buck2",
    {
      args,
      stdout: "piped",
      stderr: "piped",
    },
  ).spawn();

  // Stream stdout
  const td = new TextDecoder();
  const stdout = (async () => {
    const reader = child.stdout.getReader();
    try {
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        logger.debug(td.decode(value));
      }
    } finally {
      reader.releaseLock();
    }
  })();

  // Stream stderr
  const stderr = (async () => {
    const reader = child.stderr.getReader();
    try {
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        logger.debug(td.decode(value));
      }
    } finally {
      reader.releaseLock();
    }
  })();

  const status = await child.status;

  if (!status.success) {
    await stdout;
    await stderr;
    throw new Error(`Command failed with status: ${status.code}`);
  }

  const fullPath = Deno.realPathSync(EXISTING_PACKAGES);
  return (await import(fullPath, {
    with: { type: "json" },
  })).default;
}
