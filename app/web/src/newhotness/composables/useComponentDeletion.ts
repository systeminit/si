import { useRoute } from "vue-router";
import { ComponentId } from "@/api/sdf/dal/component";
import {
  ComponentInList,
  BifrostComponent,
  ComponentDiffStatus,
} from "@/workers/types/entity_kind_types";
import { useApi, routes } from "../api_composables";
import { DeleteMode } from "../DeleteModal.vue";

export function useComponentDeletion(viewId?: string, skipNavigation = false) {
  const route = useRoute();
  const deleteApi = useApi();
  const eraseApi = useApi();
  const deleteEraseFromViewApi = useApi();
  const restoreApi = useApi();

  const convertBifrostToComponentInList = (
    component: BifrostComponent,
  ): ComponentInList => {
    // Convert resourceDiff to diffStatus
    let diffStatus: ComponentDiffStatus;
    if (component.resourceDiff?.diff) {
      diffStatus = "Modified";
    } else if (
      component.resourceDiff?.current &&
      !component.resourceDiff?.diff
    ) {
      diffStatus = "Added";
    } else {
      diffStatus = "None";
    }

    return {
      id: component.id,
      name: component.name,
      color: component.color,
      schemaName: component.schemaName,
      schemaId: component.schemaId,
      schemaVariantId: component.schemaVariant.id,
      schemaVariantName: component.schemaVariantName,
      schemaCategory: component.schemaCategory,
      hasResource: component.hasResource,
      qualificationTotals: component.qualificationTotals,
      inputCount: component.inputCount,
      diffStatus,
      toDelete: component.toDelete,
      resourceId: null,
      hasSocketConnections: false,
    };
  };

  const deleteComponents = async (
    componentIds: ComponentId[],
    mode: DeleteMode,
  ) => {
    if (mode === DeleteMode.Delete) {
      const call = deleteApi.endpoint(routes.DeleteComponents);
      const { req, newChangeSetId } = await call.delete({
        componentIds,
        forceErase: false,
      });
      if (deleteApi.ok(req)) {
        if (newChangeSetId && !skipNavigation) {
          deleteApi.navigateToNewChangeSet(
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
      return { success: deleteApi.ok(req), newChangeSetId };
    } else if (mode === DeleteMode.Remove && viewId) {
      // Remove from view mode
      const call = deleteEraseFromViewApi.endpoint(routes.ViewEraseComponents, {
        viewId,
      });
      const { req, newChangeSetId } = await call.delete({
        componentIds,
      });
      if (deleteEraseFromViewApi.ok(req)) {
        if (newChangeSetId && !skipNavigation) {
          deleteEraseFromViewApi.navigateToNewChangeSet(
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
      return { success: deleteEraseFromViewApi.ok(req), newChangeSetId };
    }
    return { success: false, newChangeSetId: null };
  };

  const eraseComponents = async (componentIds: ComponentId[]) => {
    const call = eraseApi.endpoint(routes.DeleteComponents);
    const { req, newChangeSetId } = await call.delete({
      componentIds,
      forceErase: true,
    });

    if (eraseApi.ok(req)) {
      if (newChangeSetId && !skipNavigation) {
        eraseApi.navigateToNewChangeSet(
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
    return { success: eraseApi.ok(req), newChangeSetId };
  };

  const restoreComponents = async (componentIds: ComponentId[]) => {
    const call = restoreApi.endpoint(routes.RestoreComponents);
    const { req, newChangeSetId } = await call.put({
      componentIds,
    });

    if (restoreApi.ok(req)) {
      if (newChangeSetId && !skipNavigation) {
        restoreApi.navigateToNewChangeSet(
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
    return { success: restoreApi.ok(req), newChangeSetId };
  };

  return {
    convertBifrostToComponentInList,
    deleteComponents,
    eraseComponents,
    restoreComponents,
  };
}
