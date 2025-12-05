/**
 * Change Set Create Module - Create new change sets
 *
 * This module provides functionality to create new change sets in the
 * System Initiative workspace.
 *
 * @module
 */

import { ChangeSetsApi } from "@systeminit/api-client";
import { Context } from "../context.ts";
import { apiConfig, WORKSPACE_ID } from "../si_client.ts";
import type { ChangeSetCreateOptions } from "./types.ts";

export type { ChangeSetCreateOptions };

/**
 * Main entry point for the change-set create command
 */
export async function callChangeSetCreate(
  options: ChangeSetCreateOptions,
): Promise<void> {
  // Get context
  const ctx = Context.instance();

  try {
    if (!apiConfig || !WORKSPACE_ID) {
      throw new Error(
        "API token not found. Set SI_API_TOKEN environment variable or use --api-token flag.",
      );
    }

    const changeSetsApi = new ChangeSetsApi(apiConfig);

    const response = await changeSetsApi.createChangeSet({
      workspaceId: WORKSPACE_ID,
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
  } catch (error) {
    ctx.logger.error(`Failed to create change set: ${error}`);
    Deno.exit(1);
  }
}
