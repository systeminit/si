import assert from "node:assert";
import { SdfApiClient } from "../sdf_api_client.ts";

export default async function get_head_changeset(sdfApiClient: SdfApiClient) {
  const workspaceId = sdfApiClient.workspaceId;
  const data = await sdfApiClient.call({
    route: "list_open_change_sets",
    routeVars: {
      workspaceId,
    },
  });

  assert(data.defaultChangeSetId, "Expected headChangeSetId");
  const head = data.changeSets.find((c) => c.id === data.defaultChangeSetId);
  assert(head, "Expected a HEAD changeset");
}
