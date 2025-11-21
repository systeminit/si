import { ChangeSetsApi, type ChangeSetViewV1 } from "@systeminit/api-client";
import { Context } from "./context.ts";
import { apiConfig } from "./si_client.ts";
import type { Logger } from "@logtape/logtape";
import { AxiosError } from "axios";
import { unknownValueToErrorMessage } from "./helpers.ts";

/**
 * Resolves a change set by ID or name to its ID.
 *
 * @param workspaceId - The workspace ID
 * @param changeSetIdOrName - Either a change set ID or name
 * @returns The resolved change set ID
 * @throws Error if change set not found or name is ambiguous
 */
export async function resolveChangeSet(
  workspaceId: string,
  changeSetIdOrName: string,
): Promise<string> {
  const ctx = Context.instance();
  const changeSetsApi = new ChangeSetsApi(apiConfig);

  ctx.logger.debug(`Resolving change set: {changeSet}`, {
    changeSet: changeSetIdOrName,
  });

  const response = await changeSetsApi.listChangeSets({ workspaceId });
  const changeSets = response.data.changeSets as ChangeSetViewV1[];

  // Try exact ID match first
  const byId = changeSets.find((cs) => cs.id === changeSetIdOrName);
  if (byId) {
    ctx.logger.debug(`Found change set by ID: {id} ({name})`, {
      id: byId.id,
      name: byId.name,
    });
    return byId.id;
  }

  // Try name match
  const byName = changeSets.filter((cs) => cs.name === changeSetIdOrName);
  if (byName.length === 0) {
    throw new Error(`Change set not found: ${changeSetIdOrName}`);
  }
  if (byName.length > 1) {
    throw new Error(
      `Ambiguous change set name "${changeSetIdOrName}" matches ${byName.length} change sets`,
    );
  }

  ctx.logger.debug(`Found change set by name: {id} ({name})`, {
    id: byName[0].id,
    name: byName[0].name,
  });
  return byName[0].id;
}

// runs callback with a changeSetId, abandoning it if anything throws
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
