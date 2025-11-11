import type {
  ComponentsApi,
  UpdateComponentV1Request,
} from "@systeminit/api-client";
import { attributeDiffToUpdatePayload } from "../template/attribute_diff.ts";
import type { AttributeDiff } from "../template/converge_types.ts";
import { Context } from "../context.ts";

/**
 * Updates a component's attributes using the provided diff.
 *
 * @param api - The ComponentsApi instance
 * @param workspaceId - The workspace ID
 * @param changeSetId - The change set ID
 * @param componentId - The component ID to update
 * @param attributeDiff - The computed attribute diff to apply
 * @param nameChange - Optional name change {from: oldName, to: newName}
 */
export async function updateComponent(
  api: ComponentsApi,
  workspaceId: string,
  changeSetId: string,
  componentId: string,
  attributeDiff: AttributeDiff,
  nameChange?: { from: string; to: string },
): Promise<void> {
  const ctx = Context.instance();
  const payload = attributeDiffToUpdatePayload(attributeDiff);

  const updateRequest: UpdateComponentV1Request = {
    attributes: payload,
  };

  if (nameChange) {
    updateRequest.name = nameChange.to;
    ctx.logger.info(
      `Updating component {componentId}: name "{oldName}" -> "{newName}"`,
      {
        componentId,
        oldName: nameChange.from,
        newName: nameChange.to,
      },
    );
  } else {
    ctx.logger.info(`Updating component {componentId}`, { componentId });
  }

  ctx.logger.debug(`Update payload: {payload}`, { payload });

  await api.updateComponent({
    workspaceId,
    changeSetId,
    componentId,
    updateComponentV1Request: updateRequest,
  });

  ctx.logger.info(`Component {componentId} updated successfully`, {
    componentId,
  });
}
