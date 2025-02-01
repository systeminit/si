import _logger from "./logger.ts";
import _ from "npm:lodash";
import { PkgSpec } from "./bindings/PkgSpec.ts";

const logger = _logger.ns("packageGen").seal();
export const EXISTING_PACKAGES = "existing-packages/spec.json";

export async function getExistingSpecs(): Promise<Record<string, PkgSpec>> {
  logger.debug("Getting existing specs...");
  const td = new TextDecoder();
  const child = new Deno.Command(
    "buck2",
    {
      args: [
        "run",
        "//bin/hoist:hoist",
        "--",
        "--endpoint",
        "http://0.0.0.0:5157",
        "write-existing-modules-spec",
        "--out",
        EXISTING_PACKAGES,
      ],
      stdout: "piped",
      stderr: "piped",
    },
  ).spawn();

  // Stream stdout
  (async () => {
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
  (async () => {
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
    throw new Error(`Command failed with status: ${status.code}`);
  }

  const fullPath = Deno.realPathSync(EXISTING_PACKAGES);
  return (await import(fullPath, {
    with: { type: "json" },
  })).default;
}
