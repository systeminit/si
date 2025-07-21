import { useRoute } from "vue-router";
import { ComponentId } from "@/api/sdf/dal/component";
import { useApi, routes } from "../api_composables";

export function useComponentUpgrade() {
  const route = useRoute();
  const upgradeApi = useApi();

  const upgradeComponents = async (componentIds: ComponentId[]) => {
    const call = upgradeApi.endpoint(routes.UpgradeComponents);
    const { req, newChangeSetId } = await call.post({
      componentIds,
    });

    if (upgradeApi.ok(req)) {
      if (newChangeSetId) {
        upgradeApi.navigateToNewChangeSet(
          {
            name: "new-hotness",
            params: {
              workspacePk: route.params.workspacePk,
              changeSetId: newChangeSetId,
            },
          },
          newChangeSetId,
        );
      }
    }

    return { success: upgradeApi.ok(req), newChangeSetId };
  };

  return {
    upgradeComponents,
  };
}
