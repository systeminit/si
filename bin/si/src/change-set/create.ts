/**
 * Change Set Create Module - Create new change sets
 *
 * This module provides functionality to create new change sets in the
 * System Initiative workspace.
 *
 * @module
 */

import { open } from "jsr:@opensrc/deno-open";
import { ChangeSetsApi } from "@systeminit/api-client";
import { Context } from "../context.ts";
import type { ChangeSetCreateOptions } from "./types.ts";
import { generateChangeSetUrl } from "../helpers.ts";
import { getWorkspaceDetails } from "../cli/helpers.ts";

export type { ChangeSetCreateOptions };

/**
 * Main entry point for the change-set create command
 */
export async function callChangeSetCreate(
  options: ChangeSetCreateOptions,
): Promise<void> {
  const ctx = Context.instance();

  try {
    const apiConfig = Context.apiConfig();
    const workspaceId = Context.workspaceId();

    const changeSetsApi = new ChangeSetsApi(apiConfig);

    const response = await changeSetsApi.createChangeSet({
      workspaceId,
      createChangeSetV1Request: {
        changeSetName: options.name,
      },
    });

    const changeSet = response.data.changeSet;

    ctx.logger.info("Change set created: {*}", {
      id: changeSet.id,
      name: changeSet.name,
      status: changeSet.status,
    });

    // Open the change set in browser if --open flag is provided
    if (options.open) {
      const workspace = await getWorkspaceDetails(workspaceId);
      const baseUrl = generateChangeSetUrl(workspace, changeSet.id);
      const url = `${baseUrl}/h`;

      if (!ctx.isInteractive) {
        ctx.logger.error("Can't open browser from non-interactive shell. Visit {url} in your browser instead.", { url });
      } else {
        ctx.logger.info("Opening change set in browser: {url}", { url });
        await open(url);
        ctx.logger.info("Change set opened successfully");
      }
    }
  } catch (error) {
    ctx.logger.error(`Failed to create change set: ${error}`);
    Deno.exit(1);
  }
}
