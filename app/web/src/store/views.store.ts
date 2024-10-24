import { defineStore } from "pinia";
import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { IRect, Vector2d } from "konva/lib/types";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import {
  ViewId,
  View,
  Components,
  Sockets,
  Edges,
  Groups,
} from "@/api/sdf/dal/views";
import {
  DiagramElementUniqueKey,
  DiagramGroupData,
  DiagramNodeData,
} from "@/components/ModelingDiagram/diagram_types";
import { ComponentId, RawComponent, RawEdge } from "@/api/sdf/dal/component";
import {
  GROUP_BOTTOM_INTERNAL_PADDING,
  GROUP_INTERNAL_PADDING,
} from "@/components/ModelingDiagram/diagram_constants";
import handleStoreError from "./errors";

import { useChangeSetsStore } from "./change_sets.store";
import { useComponentsStore } from "./components.store";
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

  // implements "recent -> older" ordering while looping
  *[Symbol.iterator](): Iterator<T> {
    for (const i of this.items.reverse()) yield i;
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
  const componentsStore = useComponentsStore(forceChangeSetId);

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
        groups: {} as Groups,
        edges: {} as Edges,
        // DiagramNodeSocket can find isConnected here, so it doesn't re-render with every drag
        edgeIds: new Set() as Set<DiagramElementUniqueKey>,
        sockets: {} as Sockets,
      }),
      getters: {
        selectedView: (state) => state.viewsById[state.selectedViewId || ""],
        // NOTE: this is computed for now, but we could easily make this state
        // and re-compute it for only which elements get moved (if it becomes a bottleneck)
        contentBoundingBoxesByGroupId(state): Record<ComponentId, IRect> {
          const boxDictionary: Record<string, IRect> = {};
          const groups = Object.keys(state.groups)
            .map((c) => componentsStore.groupsById[c.substring(2)])
            .filter((c): c is DiagramGroupData => !!c);

          for (const group of groups) {
            const childIds = group.def.childIds;
            if (!childIds) continue;

            let top;
            let bottom;
            let left;
            let right;

            for (const childId of childIds) {
              const geometry =
                state.groups[childId] || state.components[childId];
              if (!geometry) continue;

              if (!top || geometry.y < top) top = geometry.y;

              const thisLeft = geometry.x - geometry.width / 2;
              if (!left || thisLeft < left) left = thisLeft;

              const thisRight = geometry.x + geometry.width / 2;
              if (!right || thisRight > right) right = thisRight;

              const thisBottom = geometry.y + geometry.height;
              if (!bottom || thisBottom > bottom) bottom = thisBottom;
            }

            if (
              left === undefined ||
              right === undefined ||
              top === undefined ||
              bottom === undefined
            )
              continue;

            // i dont know if i need these paddings yet
            boxDictionary[group.def.id] = {
              x: left - GROUP_INTERNAL_PADDING,
              y: top - GROUP_INTERNAL_PADDING,
              width: right - left + GROUP_INTERNAL_PADDING * 2,
              height:
                bottom -
                top +
                GROUP_INTERNAL_PADDING +
                GROUP_BOTTOM_INTERNAL_PADDING,
            };
          }

          return boxDictionary;
        },
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
            this.groups = view.groups;
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
        async FETCH_VIEW(viewId?: ViewId) {
          // TODO, fetch, and set to selected view
          return new ApiRequest<{
            viewId: ViewId;
            components: RawComponent[];
            edges: RawEdge[];
            inferredEdges: RawEdge[];
          }>({
            url: "diagram/get_diagram",
            params: {
              ...visibilityParams,
              viewId,
            },
            onSuccess: (response) => {
              componentsStore.SET_COMPONENTS_FROM_VIEW(response);
              const components: DiagramNodeData[] = [];
              const groups: DiagramGroupData[] = [];
              for (const component of response.components) {
                // doing this to piggy back on the position data, but it will change with Victor's changes!
                const c = componentsStore.allComponentsById[component.id];
                if (c) {
                  if (c.def.isGroup) groups.push(c as DiagramGroupData);
                  else components.push(c as DiagramNodeData);
                }
              }
              this.SET_COMPONENTS_FROM_VIEW(response.viewId, {
                components,
                groups,
              });
              this.selectView(response.viewId);

              // fire this and don't wait for it
              componentsStore.FETCH_ALL_COMPONENTS();
              // load all other view geometry
            },
          });
        },
        SET_COMPONENTS_FROM_VIEW(
          viewId: ViewId,
          response: {
            components: DiagramNodeData[];
            groups: DiagramGroupData[];
            edges?: RawEdge[];
            inferredEdges?: RawEdge[];
          },
        ) {
          const components: Components = {};
          const groups: Groups = {};
          for (const component of response.components) {
            const geo = { ...component.def.position } as IRect;
            geo.width = component.width;
            geo.height = component.height;
            components[component.def.id] = geo;
          }
          for (const group of response.groups) {
            groups[group.def.id] = {
              ...group.def.position,
              // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
              ...group.def.size!,
            };
          }
          // TODO
          this.viewsById[viewId] = { components, groups };
        },
        // REDO the 409 conflicts and retry logic
        async MOVE_COMPONENTS(
          clientUlid: string,
          components: (DiagramGroupData | DiagramNodeData)[],
          positionDelta: Vector2d,
          opts: { writeToChangeSet?: boolean; broadcastToClients?: boolean },
        ) {
          // TODO, bump all elements and their sockets by the vector
        },
        async RESIZE_COMPONENT(
          clientUlid: string,
          component: DiagramGroupData,
          geometry: IRect,
          opts: { writeToChangeSet?: boolean; broadcastToClients?: boolean },
        ) {
          // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
          const origGeometry = structuredClone(this.groups[component.def.id]!);
          const delta: Vector2d = {
            x: origGeometry.x - geometry.x,
            y: origGeometry.y - geometry.y,
          };

          this.groups[component.def.id] = { ...geometry };

          if (delta.x !== 0 || delta.y !== 0) {
            // TODO, sockets need to be re-positioned if delta X or Y is not 0
          }
          // TODO, save, broadcast
        },
      },
      onActivated() {
        if (!changeSetId) return;
        this.FETCH_VIEW();
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
                  this.FETCH_VIEW();
                  this.LIST_VIEWS();
                  // LOAD ALL OTHER VIEW DATA, if its dirty
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
  )();
};
