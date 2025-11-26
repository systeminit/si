/**
 * Schema-specific change set utilities
 *
 * This module provides change set wrapping functionality used by schema push operations.
 */

import type { ChangeSetsApi } from "@systeminit/api-client";
import type { Logger } from "@logtape/logtape";
import { AxiosError } from "axios";
import { unknownValueToErrorMessage } from "../helpers.ts";

/**
 * Runs callback with a changeSetId, abandoning it if anything throws
 *
 * @param api - ChangeSetsApi instance
 * @param logger - Logger instance
 * @param workspaceId - The workspace ID
 * @param changeSetNamePrefix - Prefix for the change set name
 * @param callback - Async function to execute with the change set ID
 */
export async function wrapInChangeSet(
  api: ChangeSetsApi,
  logger: Logger,
  workspaceId: string,
  changeSetNamePrefix: string,
  callback: (changeSetId: string) => Promise<void>,
) {
  const changeSetName = changeSetNamePrefix + " " + new Date().toISOString();

  const createChangeSetResponse = await api.createChangeSet({
    workspaceId,
    createChangeSetV1Request: { changeSetName },
  });

  const changeSetId = createChangeSetResponse.data.changeSet.id;

  try {
    await callback(changeSetId);
  } catch (error) {
    if (error instanceof AxiosError) {
      logger.error(
        `API error on: (${error.status}) ${error.response?.data.message}`,
      );
      logger.error(`Request: ${error.request.method} ${error.request.path}`);
    } else {
      logger.error(
        `Error creating schemas: ${unknownValueToErrorMessage(error)}`,
      );
    }
    logger.info("Deleting change set...");
    api.abandonChangeSet({
      workspaceId,
      changeSetId,
    });
  }
}
