import _logger from "./logger.ts";
import _ from "npm:lodash";
import { PkgSpec } from "./bindings/PkgSpec.ts";
import { emptyDirectory } from "./util.ts";

const logger = _logger.ns("packageGen").seal();
export const EXISTING_PACKAGES = "existing-packages";

export async function getExistingSpecs(): Promise<Record<string, PkgSpec>> {
  await emptyDirectory(EXISTING_PACKAGES);
  logger.debug("Getting existing specs...");
  const td = new TextDecoder();
  const child = await new Deno.Command(
    "buck2",
    {
      args: [
        "run",
        "//bin/hoist:hoist",
        "--",
        "--endpoint",
        "http://0.0.0.0:5157",
        "write-all-specs",
        "--out",
        EXISTING_PACKAGES,
      ],
    },
  ).output();

  const stdout = td.decode(child.stdout).trim();
  logger.debug(stdout);
  const stderr = td.decode(child.stderr).trim();
  logger.debug(stderr);

  const fullPath = Deno.realPathSync(EXISTING_PACKAGES);

  const existing_specs: Record<string, PkgSpec> = {};

  for (const dirEntry of Deno.readDirSync(fullPath)) {
    if (!dirEntry.name.includes(".json")) continue;

    const filename = `${fullPath}/${dirEntry.name}`;

    const rawData = await import(filename, {
      with: { type: "json" },
    });
    const existing_spec = rawData.default as PkgSpec;
    existing_specs[existing_spec.name] = existing_spec;
  }

  return existing_specs;
}
