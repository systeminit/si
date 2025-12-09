/**
 * Change Set Open Module - Open change sets in browser
 *
 * This module provides functionality to open a change set in the
 * System Initiative web application in the default browser.
 *
 * @module
 */

import { open } from "@opensrc/deno-open";
import { Context } from "../context.ts";
import { WORKSPACE_ID } from "../si_client.ts";
import type { ChangeSetOpenOptions } from "./types.ts";
import { resolveChangeSet } from "./utils.ts";
import { initializeCliContextWithAuth } from "../cli/helpers.ts";
import { WorkspaceDetails } from "../cli/auth.ts";
import { generateChangeSetUrl } from "../helpers.ts";

export type { ChangeSetOpenOptions };

/**
 * Main entry point for the change-set open command
 */
export async function callChangeSetOpen(
  options: ChangeSetOpenOptions,
): Promise<void> {
  const ctx = Context.instance();
  const { workspace } = await initializeCliContextWithAuth({ ctx: Context.instance() });

  try {
    if (!WORKSPACE_ID) {
      throw new Error(
        "API token not found. Set SI_API_TOKEN environment variable or use --api-token flag.",
      );
    }

    // Resolve the change set ID from the provided ID or name
    const changeSetId = await resolveChangeSet(
      WORKSPACE_ID,
      options.changeSetIdOrName,
    );

    // Generate the URL
    const url = generateChangeSetUrl(workspace, changeSetId);

    ctx.logger.info("Opening change set in browser: {url}", { url });

    // Open in browser using deno-open
    await open(`${url}/h`);

    ctx.logger.info("Change set opened successfully");
  } catch (error) {
    ctx.logger.error(`Failed to open change set: ${error}`);
    Deno.exit(1);
  }
}
