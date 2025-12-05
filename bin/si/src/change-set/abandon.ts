/**
 * Change Set Abandon Module - Abandon (delete) change sets
 *
 * This module provides functionality to abandon change sets in the
 * System Initiative workspace. Note that HEAD change sets cannot be abandoned.
 *
 * @module
 */

import { ChangeSetsApi } from "@systeminit/api-client";
import { Context } from "../context.ts";
import { apiConfig, WORKSPACE_ID } from "../si_client.ts";
import type { ChangeSetAbandonOptions } from "./types.ts";
import { resolveChangeSet } from "./utils.ts";

export type { ChangeSetAbandonOptions };

/**
 * Main entry point for the change-set abandon command
 */
export async function callChangeSetAbandon(
  options: ChangeSetAbandonOptions,
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

    // Resolve the change set ID from the provided ID or name
    const changeSetId = await resolveChangeSet(
      WORKSPACE_ID,
      options.changeSetIdOrName,
    );

    // Check if the change set is HEAD - cannot abandon HEAD
    const getResponse = await changeSetsApi.getChangeSet({
      workspaceId: WORKSPACE_ID,
      changeSetId,
    });

    if (getResponse.data.changeSet.isHead) {
      throw new Error(
        "Cannot abandon HEAD change set. HEAD is immutable and cannot be deleted.",
      );
    }

    // Abandon the change set
    const response = await changeSetsApi.abandonChangeSet({
      workspaceId: WORKSPACE_ID,
      changeSetId,
    });

    if (response.data.success) {
      ctx.logger.info("Change set abandoned: {*}", {
        id: changeSetId,
        name: getResponse.data.changeSet.name,
      });
    }
  } catch (error) {
    ctx.logger.error(`Failed to abandon change set: ${error}`);
    Deno.exit(1);
  }
}
