import { defineStore } from "pinia";
import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { ViewId, View, Components, Sockets, Edges } from "@/api/sdf/dal/views";
import handleStoreError from "./errors";

import { useChangeSetsStore } from "./change_sets.store";
import { useRealtimeStore } from "./realtime/realtime.store";
import { useWorkspacesStore } from "./workspaces.store";

/**
 * In general we treat the front end POSITION data as truth
 * And push it to the backend, retries, last wins, etc
 */
export const useViewsStore = (forceChangeSetId?: ChangeSetId) => {
  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspacePk;
  const changeSetsStore = useChangeSetsStore();

  let changeSetId: ChangeSetId | undefined;
  if (forceChangeSetId) {
    changeSetId = forceChangeSetId;
  } else {
    changeSetId = changeSetsStore.selectedChangeSetId;
  }

  const visibilityParams = {
    visibility_change_set_pk: changeSetId,
    workspaceId,
  };

  return addStoreHooks(
    workspaceId,
    changeSetId,
    defineStore(`ws${workspaceId || "NONE"}/cs${changeSetId || "NONE"}/views`, {
      state: () => ({
        selectedViewId: null as ViewId | null,

        // every views data goes here
        viewsById: {} as Record<ViewId, View>,

        /* *
         * these hold the data for everything on the diagram in the SELECTED view
         * as selectedView changes we repopulate all this data
         * this is pushing together `movedElementPositions` and `resizedElementSizes`
         * and can make `renderedGeometriesByComponentId` unnecessary
         * */
        components: {} as Components,
        edges: {} as Edges,
        sockets: {} as Sockets,
      }),
      getters: {
        selectedView: (state) => state.viewsById[state.selectedViewId || ""],
      },
      actions: {
        selectView(id: ViewId) {
          const view = this.viewsById[id];
          if (view) {
            this.selectedViewId = id;
            /* *
             * i think i want to set these as in-memory references
             * that way i don't have to do two writes for incoming WsEvents
             * or two writes for user actions
             *
             * this does mean that `draggedElementsPositionsPreDrag` and
             * `resizedElementSizesPreResize` need to be populated
             * but those could just be a `structuredClone` of this data
             * */
            this.components = view.components;
            this.edges = view.edges;
            this.sockets = view.sockets;
          }
        },
      },
      onActivated() {
        if (!changeSetId) return;

        const realtimeStore = useRealtimeStore();

        realtimeStore.subscribe(
          `${this.$id}-changeset`,
          `changeset/${changeSetId}`,
          [],
        );

        realtimeStore.subscribe(
          `${this.$id}-workspace`,
          `workspace/${workspaceId}`,
          [
            {
              eventType: "ChangeSetApplied",
              callback: (data) => {
                // If the applied change set has rebased into this change set,
                // then refetch (i.e. there might be updates!)
                if (data.toRebaseChangeSetId === changeSetId) {
                }
              },
            },
          ],
        );

        const actionUnsub = this.$onAction(handleStoreError);

        return () => {
          realtimeStore.unsubscribe(`${this.$id}-changeset`);
          realtimeStore.unsubscribe(`${this.$id}-workspace`);
        };
      },
    }),
  );
};
