/**
 * Change Set Open Module - Open change sets in browser
 *
 * This module provides functionality to open a change set in the
 * System Initiative web application in the default browser.
 *
 * @module
 */

import { open } from "jsr:@opensrc/deno-open";
import { Context } from "../context.ts";
import type { ChangeSetOpenOptions } from "./types.ts";
import { resolveChangeSet } from "./utils.ts";
import { generateChangeSetUrl } from "../helpers.ts";
import { getWorkspaceDetails } from "../cli/helpers.ts";

export type { ChangeSetOpenOptions };

/**
 * Main entry point for the change-set open command
 */
export async function callChangeSetOpen(
  options: ChangeSetOpenOptions,
): Promise<void> {
  const ctx = Context.instance();
  const workspaceId = Context.workspaceId();
  const workspace = await getWorkspaceDetails(workspaceId);

  try {
    // Resolve the change set ID from the provided ID or name
    const changeSetId = await resolveChangeSet(
      workspaceId,
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
