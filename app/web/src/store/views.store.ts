import { defineStore } from "pinia";
import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { IRect, Vector2d } from "konva/lib/types";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { ViewId, View, Components, Sockets, Edges } from "@/api/sdf/dal/views";
import { DiagramElementUniqueKey } from "@/components/ModelingDiagram/diagram_types";
import handleStoreError from "./errors";

import { useChangeSetsStore } from "./change_sets.store";
import { useRealtimeStore } from "./realtime/realtime.store";
import { useWorkspacesStore } from "./workspaces.store";

class UniqueStack<T> {
  items: T[];

  constructor() {
    this.items = [];
  }

  idx(i: T) {
    return this.items.findIndex((_i) => _i === i);
  }

  push(i: T) {
    if (this.idx(i) !== -1) this.items.push(i);
  }

  remove(i: T) {
    const idx = this.idx(i);
    if (idx !== -1) this.items.splice(idx, 1);
  }

  pop() {
    if (this.items.length === 0) {
      return null;
    }
    return this.items.pop();
  }
}

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
        recentViews: new UniqueStack() as UniqueStack<ViewId>,

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
        // DiagramNodeSocket can find isConnected here, so it doesn't re-render with every drag
        edgeIds: new Set() as Set<DiagramElementUniqueKey>,
        sockets: {} as Sockets,
      }),
      getters: {
        selectedView: (state) => state.viewsById[state.selectedViewId || ""],
      },
      actions: {
        selectView(id: ViewId) {
          const view = this.viewsById[id];
          if (view) {
            // move the currently selected view to the top of the
            if (this.selectedViewId) {
              this.pushRecentView(this.selectedViewId);
            }
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
            // currently edges store their socket location information
            // internally... maybe we should stop that
            this.edges = view.edges;
            this.edgeIds = new Set(Object.keys(view.edges));
            // derive the socket position from the component position
            // to begin, and then adjust it via delta when things move
            this.sockets = view.sockets;
          }
        },
        closeRecentView(id: ViewId) {
          this.recentViews.remove(id);
        },
        pushRecentView(id: ViewId) {
          this.recentViews.push(id);
        },
        async LIST_VIEWS() {
          // TODO
        },
        // no viewId means load the default
        async SELECT_VIEW(viewId?: ViewId) {
          // TODO, fetch, and set to selected view
        },
        async MOVE_COMPONENTS(
          components: DiagramElementUniqueKey[],
          positionDelta: Vector2d,
          writeToChangeSet?: boolean,
          broadcastToClients?: boolean,
        ) {
          // TODO, bump all elements and their sockets by the vector
        },
        async RESIZE_COMPONENT(
          component: DiagramElementUniqueKey,
          position: IRect,
          positionDelta: Vector2d,
          writeToChangeSet?: boolean,
          broadcastToClients?: boolean,
        ) {
          // TODO, sockets need to be re-positioned if delta X or Y is not 0
        },
      },
      onActivated() {
        if (!changeSetId) return;
        this.SELECT_VIEW();
        this.LIST_VIEWS();

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
