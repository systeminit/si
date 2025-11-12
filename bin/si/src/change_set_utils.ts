import { ChangeSetsApi, type ChangeSetViewV1 } from "@systeminit/api-client";
import { Context } from "./context.ts";
import { apiConfig } from "./si_client.ts";

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
