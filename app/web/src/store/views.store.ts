import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { addStoreHooks, ApiRequest, URLPattern } from "@si/vue-lib/pinia";
import { IRect, Vector2d } from "konva/lib/types";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import {
  ViewId,
  View,
  Components,
  Sockets,
  Groups,
  ViewDescription,
  StringGeometry,
} from "@/api/sdf/dal/views";
import {
  DiagramGroupData,
  DiagramNodeData,
  Size2D,
} from "@/components/ModelingDiagram/diagram_types";
import { ComponentId, RawComponent, RawEdge } from "@/api/sdf/dal/component";
import {
  GROUP_BOTTOM_INTERNAL_PADDING,
  GROUP_INTERNAL_PADDING,
  NODE_WIDTH,
  SOCKET_SIZE,
} from "@/components/ModelingDiagram/diagram_constants";
import { vectorAdd } from "@/components/ModelingDiagram/utils/math";
import { DefaultMap } from "@/utils/defaultmap";
import { ComponentType, SchemaVariant } from "@/api/sdf/dal/schema";
import { nonNullable } from "@/utils/typescriptLinter";
import handleStoreError from "./errors";

import { useChangeSetsStore } from "./change_sets.store";
import { useComponentsStore, processRawComponent } from "./components.store";
import { useRealtimeStore } from "./realtime/realtime.store";
import { useWorkspacesStore } from "./workspaces.store";
import { useAssetStore } from "./asset.store";

const MAX_RETRIES = 5;

type PendingComponent = {
  tempId: string;
  position: Vector2d;
};

type RequestUlid = string;

class UniqueStack<T> {
  items: T[];

  constructor() {
    this.items = [];
  }

  idx(i: T) {
    return this.items.findIndex((_i) => _i === i);
  }

  push(i: T) {
    if (this.idx(i) === -1) this.items.push(i);
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

const setSockets = (
  component: DiagramGroupData | DiagramNodeData,
  geometry: Vector2d & Partial<Size2D>,
) => {
  const sockets: Sockets = {};
  const width = geometry.width || NODE_WIDTH;

  const left = component.layoutLeftSockets(width);
  left.sockets.forEach((s) => {
    const center = {
      x: geometry.x + left.x + s.position.x - SOCKET_SIZE,
      y: geometry.y + left.y + s.position.y,
    };
    sockets[s.uniqueKey] = {
      center,
    };
  });
  const right = component.layoutRightSockets(width);
  right.sockets.forEach((s) => {
    const center = {
      x:
        geometry.x +
        right.x +
        width - // we add the full width to get to the right side...
        SOCKET_SIZE + // minus the size of the socket, b/c later code adds half the size of the socket
        s.position.x,
      y: geometry.y + right.y + s.position.y,
    };
    sockets[s.uniqueKey] = {
      center,
    };
  });
  return sockets;
};

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

  const API_PREFIX = [
    "v2",
    "workspaces",
    { workspaceId },
    "change-sets",
    { changeSetId },
    "views",
  ] as URLPattern;

  return addStoreHooks(
    workspaceId,
    changeSetId,
    defineStore(`ws${workspaceId || "NONE"}/cs${changeSetId || "NONE"}/views`, {
      state: () => ({
        diagramUlid: "",
        selectedViewId: null as ViewId | null,
        recentViews: new UniqueStack() as UniqueStack<ViewId>,

        // every views data goes here
        viewsById: {} as Record<ViewId, View>,
        viewList: [] as ViewDescription[],

        /* *
         * these hold the data for everything on the diagram in the SELECTED view
         * as selectedView changes we repopulate all this data
         * this is pushing together `movedElementPositions` and `resizedElementSizes`
         * and can make `renderedGeometriesByComponentId` unnecessary
         * */
        components: {} as Components,
        groups: {} as Groups,
        sockets: {} as Sockets,

        // size of components when dragged to the stage
        inflightElementSizes: {} as Record<RequestUlid, ComponentId[]>,
        // prevents run away retries, unknown what circumstances could lead to this, but protecting ourselves
        inflightRetryCounter: new DefaultMap<string, number>(() => 0),
        pendingInsertedComponents: {} as Record<string, PendingComponent>,
      }),
      getters: {
        diagramIsEmpty(state): boolean {
          return (
            Object.keys(state.components).length === 0 &&
            Object.keys(state.groups).length === 0
          );
        },

        defaultViewId: (state) => {
          const idx = state.viewList.findIndex((v) => v.isDefault);
          return idx !== -1 ? state.viewList[idx]?.id : state.viewList[0]?.id;
        },
        edges: (state) =>
          Object.values(componentsStore.diagramEdgesById).filter((e) => {
            const to = e.toNodeKey.substring(2);
            const from = e.fromNodeKey.substring(2);
            const componentIds = Object.keys(state.components).concat(
              Object.keys(state.groups),
            );
            return componentIds.includes(to) && componentIds.includes(from);
          }),
        selectedView: (state) => state.viewsById[state.selectedViewId || ""],
        // NOTE: this is computed for now, but we could easily make this state
        // and re-compute it for only which elements get moved (if it becomes a bottleneck)
        contentBoundingBoxesByGroupId(state): Record<ComponentId, IRect> {
          const boxDictionary: Record<string, IRect> = {};
          const groups = Object.keys(state.groups)
            .map((c) => componentsStore.groupsById[c])
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
        async selectView(id: ViewId) {
          const view = this.viewsById[id];
          if (view) {
            // move the currently selected view to the top of the
            if (this.selectedViewId) {
              this.pushRecentView(this.selectedViewId);
            }
            /* if (!Object.keys(this.viewsById).includes(id)) {
              await this.FETCH_VIEW(id);
              if (!Object.keys(this.viewsById).includes(id))
                throw new Error(`${id} does not exist`);
            } */
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
          return new ApiRequest<ViewDescription[]>({
            method: "get",
            url: API_PREFIX,
            onSuccess: (views) => {
              this.viewList = views;
            },
          });
        },
        async CREATE_VIEW(name: string) {
          return new ApiRequest<ViewDescription>({
            method: "post",
            url: API_PREFIX,
            params: { name },
            onSuccess: (view) => {
              const idx = this.viewList.findIndex((v) => v.name === name);
              // confirming we dont already have the data
              if (idx === -1) this.viewList.push(view);
            },
          });
        },
        async UPDATE_VIEW_NAME(view_id: ViewId, name: string) {
          return new ApiRequest<null>({
            method: "put",
            url: API_PREFIX.concat([view_id]),
            params: { name },
            optimistic: () => {
              const v = this.viewList.find((v) => v.id === view_id);
              if (v) v.name = name;
              const _v = this.viewsById[view_id];
              if (_v) _v.name = name;
            },
          });
        },

        async FETCH_VIEW(viewId?: ViewId) {
          let url = API_PREFIX.concat(["default", "get_diagram"]);
          if (viewId) url = API_PREFIX.concat([{ viewId }, "get_diagram"]);

          return new ApiRequest<{
            view: ViewDescription;
            diagram: {
              components: RawComponent[];
              edges: RawEdge[];
              inferredEdges: RawEdge[];
            };
          }>({
            url,
            params: {
              ...visibilityParams,
            },
            onSuccess: (response) => {
              componentsStore.SET_COMPONENTS_FROM_VIEW(response.diagram);
              const components: RawComponent[] = [];
              const groups: RawComponent[] = [];
              for (const component of response.diagram.components) {
                if (component) {
                  if (component.componentType === ComponentType.Component)
                    components.push(component);
                  else groups.push(component);
                }
              }
              this.SET_COMPONENTS_FROM_VIEW(response.view, {
                components,
                groups,
              });
              this.selectView(response.view.id);

              // fire this and don't wait for it
              componentsStore.FETCH_ALL_COMPONENTS();
              // load all other view geometry
            },
          });
        },
        SET_COMPONENTS_FROM_VIEW(
          view: ViewDescription,
          response: {
            components: RawComponent[];
            groups: RawComponent[];
          },
        ) {
          const components: Components = {};
          const groups: Groups = {};
          const sockets: Sockets = {};
          for (const component of response.components) {
            if (!component.viewData?.geometry) continue;
            // Note: its actually a Vector2D, but type guarding it so i can set the width and height below
            const geo = { ...component.viewData.geometry } as IRect;
            // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
            const node = componentsStore.allComponentsById[
              component.id
            ]! as DiagramNodeData;
            geo.width = node.width;
            geo.height = node.height;
            components[component.id] = geo;
            for (const [key, loc] of Object.entries(setSockets(node, geo))) {
              sockets[key] = loc;
            }
          }
          for (const group of response.groups) {
            const geo = group.viewData?.geometry;
            if (!geo) continue;
            const geometry: IRect = { ...(geo as IRect) };
            if (!geometry.width) geometry.width = 500;
            if (!geometry.height) geometry.height = 500;
            groups[group.id] = {
              // this one is actually an IRect
              ...geometry,
            };
            // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
            const node = componentsStore.allComponentsById[
              group.id
            ]! as DiagramGroupData;
            for (const [key, loc] of Object.entries(
              setSockets(node, geometry),
            )) {
              sockets[key] = loc;
            }
          }
          this.viewsById[view.id] = {
            id: view.id,
            name: view.name,
            components,
            groups,
            sockets,
          };
        },
        /**
         * @param clientUlid whoami
         * @param components the selected components acted upon
         * @param positionDelta the vector to adjust all elements
         * @param opts broadcast and persistence
         */
        // REDO the 409 conflicts and retry logic
        async MOVE_COMPONENTS(
          clientUlid: string,
          components: (DiagramGroupData | DiagramNodeData)[],
          positionDelta: Vector2d,
          opts: { writeToChangeSet?: boolean; broadcastToClients?: boolean },
        ) {
          this.diagramUlid = clientUlid;
          if (positionDelta.x !== 0 || positionDelta.y !== 0) {
            components.forEach((c) => {
              const orig = c.def.isGroup
                ? this.groups[c.def.id]
                : this.components[c.def.id];
              if (!orig) return;

              const newPos = vectorAdd({ ...orig }, positionDelta);
              orig.x = newPos.x;
              orig.y = newPos.y;

              c.sockets.forEach((s) => {
                const geo = this.sockets[s.uniqueKey];
                if (!geo) return;
                geo.center.x += positionDelta.x;
                geo.center.y += positionDelta.y;
              });
            });
          }
          if (!opts.broadcastToClients && !opts.writeToChangeSet) return;

          if (opts.broadcastToClients && changeSetsStore.selectedChangeSetId) {
            const payload: Record<ComponentId, IRect> = {};
            components.forEach((c) => {
              const geo = c.def.isGroup
                ? this.groups[c.def.id]
                : this.components[c.def.id];
              if (geo) {
                if (c.def.isGroup) {
                  if (!geo.width) geo.width = 500;
                  if (!geo.height) geo.height = 500;
                }
                payload[c.def.id] = {
                  x: Math.round(geo.x),
                  y: Math.round(geo.y),
                  width: Math.round(geo.width),
                  height: Math.round(geo.height),
                };
              }
            });
            const realtimeStore = useRealtimeStore();
            const positions = Object.entries(payload).map(
              ([componentId, geo]) => ({ componentId, ...geo }),
            );
            realtimeStore.sendMessage({
              kind: "ComponentSetPosition",
              data: {
                positions,
                // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
                viewId: this.selectedViewId!,
                clientUlid,
                changeSetId: changeSetsStore.selectedChangeSetId,
              },
            });
          }
          if (opts.writeToChangeSet) {
            const payload: Record<ComponentId, StringGeometry> = {};
            components.forEach((c) => {
              const geo = c.def.isGroup
                ? this.groups[c.def.id]
                : this.components[c.def.id];
              if (geo)
                payload[c.def.id] = {
                  x: Math.round(geo.x).toString(),
                  y: Math.round(geo.y).toString(),
                  width: Math.round(geo.width).toString(),
                  height: Math.round(geo.height).toString(),
                };
            });
            return new ApiRequest<{ requestUlid: RequestUlid }>({
              method: "put",
              url: API_PREFIX.concat([
                { viewId: this.selectedViewId },
                "component/set_geometry",
              ]),
              params: {
                clientUlid,
                dataByComponentId: payload,
                ...visibilityParams,
              },
              optimistic: (requestUlid: RequestUlid) => {
                this.inflightElementSizes[requestUlid] = Object.keys(payload);
              },
              onFail: (err) => {
                // only handle conflicts here
                if (err.response.status !== 409) {
                  return;
                }
                const reqPayload = JSON.parse(err.config.data);

                // are the components that failed currently inflight?
                // if not, resend their latest data
                const failed =
                  this.inflightElementSizes[reqPayload.requestUlid];
                if (!failed) return;
                delete this.inflightElementSizes[reqPayload.requestUlid];
                const all_inflight_components = new Set(
                  Object.values(this.inflightElementSizes).flat(),
                );

                const maybe_retry: ComponentId[] = (
                  failed as ComponentId[]
                ).filter((x) => !all_inflight_components.has(x));

                const prevent = new Set();
                for (const componentId of maybe_retry) {
                  const cnt =
                    (this.inflightRetryCounter.get(componentId) || 0) + 1;
                  if (cnt > MAX_RETRIES) prevent.add(componentId);
                  else this.inflightRetryCounter.set(componentId, cnt);
                }

                if (prevent.size > 0) throw Error("Too many retries");

                const retry = maybe_retry.filter((x) => !prevent.has(x));

                if (retry.length > 0) {
                  const components = retry
                    .map(
                      (componentId) =>
                        componentsStore.allComponentsById[componentId],
                    )
                    .filter(nonNullable);
                  this.MOVE_COMPONENTS(
                    clientUlid,
                    components,
                    { x: 0, y: 0 },
                    { writeToChangeSet: true },
                  );
                }
              },
              onSuccess: (response) => {
                delete this.inflightElementSizes[response.requestUlid];
              },
            });
          }
        },
        async RESIZE_COMPONENT(
          clientUlid: string,
          component: DiagramGroupData,
          geometry: IRect,
          opts: { writeToChangeSet?: boolean; broadcastToClients?: boolean },
        ) {
          this.diagramUlid = clientUlid;

          geometry.x = Math.round(geometry.x);
          geometry.y = Math.round(geometry.y);
          geometry.width = Math.round(geometry.width);
          geometry.height = Math.round(geometry.height);

          this.groups[component.def.id] = { ...geometry };

          for (const [key, loc] of Object.entries(
            setSockets(component, geometry),
          )) {
            this.sockets[key] = loc;
          }

          if (opts.broadcastToClients && changeSetsStore.selectedChangeSetId) {
            const realtimeStore = useRealtimeStore();
            const positions = [{ componentId: component.def.id, ...geometry }];
            realtimeStore.sendMessage({
              kind: "ComponentSetPosition",
              data: {
                positions,
                // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
                viewId: this.selectedViewId!,
                clientUlid,
                changeSetId: changeSetsStore.selectedChangeSetId,
              },
            });
          }

          if (opts.writeToChangeSet) {
            const payload: Record<ComponentId, StringGeometry> = {};
            payload[component.def.id] = {
              x: Math.round(geometry.x).toString(),
              y: Math.round(geometry.y).toString(),
              width: Math.round(geometry.width).toString(),
              height: Math.round(geometry.height).toString(),
            };
            return new ApiRequest<{ requestUlid: RequestUlid }>({
              method: "put",
              url: API_PREFIX.concat([
                { viewId: this.selectedViewId },
                "component/set_geometry",
              ]),
              params: {
                clientUlid,
                dataByComponentId: payload,
                ...visibilityParams,
              },
            });
          }
        },

        async SET_PARENT(
          clientUlid: string,
          componentIds: ComponentId[],
          newParentId: ComponentId | null,
        ) {
          this.diagramUlid = clientUlid;
          const parentIdByComponentId: Record<ComponentId, ComponentId | null> =
            {};
          componentIds.forEach((componentId) => {
            parentIdByComponentId[componentId] = newParentId;
          });
          const oldParentIds: Record<ComponentId, ComponentId | undefined> = {};
          return new ApiRequest<{ requestUlid: RequestUlid }>({
            method: "put",
            url: API_PREFIX.concat([
              { viewId: this.selectedViewId },
              "component/set_parent",
            ]),
            params: {
              ...visibilityParams,
              clientUlid,
              parentIdByComponentId,
            },
            optimistic: () => {
              componentIds.forEach((componentId) => {
                const component =
                  componentsStore.rawComponentsById[componentId];
                if (!component) return;
                oldParentIds[componentId] = component.parentId;
                if (newParentId) component.parentId = newParentId;
                else component.parentId = undefined;
                componentsStore.processRawComponent(componentId, true);
              });
            },
            onFail: () => {
              componentIds.forEach((componentId) => {
                const component =
                  componentsStore.rawComponentsById[componentId];
                if (!component) return;
                component.parentId = oldParentIds[componentId];
                componentsStore.processRawComponent(componentId, true);
              });
            },
          });
        },

        async CREATE_COMPONENT(
          categoryVariantId: string,
          position: Vector2d,
          parentId?: string,
          size?: Size2D,
        ) {
          if (changeSetsStore.creatingChangeSet)
            throw new Error("race, wait until the change set is created");
          if (changeSetId === changeSetsStore.headChangeSetId)
            changeSetsStore.creatingChangeSet = true;

          const categoryVariant =
            componentsStore.categoryVariantById[categoryVariantId];
          if (!categoryVariant) {
            return;
          }

          const idAndType =
            categoryVariant.type === "installed"
              ? {
                  schemaType: "installed",
                  schemaVariantId: categoryVariant.variant.schemaVariantId,
                }
              : {
                  schemaType: "uninstalled",
                  schemaId: categoryVariant.variant.schemaId,
                };

          const tempInsertId = _.uniqueId("temp-insert-component");

          return new ApiRequest<{
            componentId: ComponentId;
            installedVariant?: SchemaVariant;
          }>({
            method: "post",
            url: API_PREFIX.concat([
              { viewId: this.selectedViewId },
              "component",
            ]),
            headers: { accept: "application/json" },
            params: {
              parentId,
              x: Math.round(position.x).toString(),
              y: Math.round(position.y).toString(),
              height: Math.round(size?.height ?? 0).toString(),
              width: Math.round(size?.width ?? 0).toString(),
              ...idAndType,
              ...visibilityParams,
            },
            optimistic: () => {
              /**
               * NOTE: WsEvents are firing *BEFORE* the POST returns
               * And when a new change set is created by the backend, we end up
               * re-creating componentStore, so the store for HEAD never runs onSuccess below
               * We end up with pending components on HEAD that never go away
               *
               * To fix: don't create pending components if we're on HEAD
               */
              if (changeSetsStore.headSelected) return;

              this.pendingInsertedComponents[tempInsertId] = {
                tempId: tempInsertId,
                position,
              };

              return () => {
                delete this.pendingInsertedComponents[tempInsertId];
              };
            },
            onSuccess: (response) => {
              delete this.pendingInsertedComponents[tempInsertId];
              if (
                categoryVariant.type === "uninstalled" &&
                response.installedVariant
              ) {
                const assetStore = useAssetStore();
                const installedVariant = response.installedVariant;
                assetStore.uninstalledVariantList =
                  assetStore.uninstalledVariantList.filter(
                    (variant) => variant.schemaId !== installedVariant.schemaId,
                  );
                assetStore.schemaVariants.push(installedVariant);
              }
            },
          });
        },

        async PASTE_COMPONENTS(
          components: {
            id: ComponentId;
            componentGeometry: Vector2d & Size2D;
          }[],
          newParentNodeId?: ComponentId,
        ) {
          if (changeSetsStore.creatingChangeSet)
            throw new Error("race, wait until the change set is created");
          if (changeSetId === changeSetsStore.headChangeSetId)
            changeSetsStore.creatingChangeSet = true;

          if (components.length === 0) return;

          const tempInserts = _.map(components, (c) => ({
            id: _.uniqueId("temp-insert-component"),
            position: {
              x: c.componentGeometry.x,
              y: c.componentGeometry.y + c.componentGeometry.height / 2,
            },
          }));

          for (const { id: tempId, position } of tempInserts) {
            this.pendingInsertedComponents[tempId] = {
              tempId,
              position,
            };
          }

          const APIComponents = _.map(components, (c) => ({
            id: c.id,
            componentGeometry: {
              x: Math.round(c.componentGeometry.x).toString(),
              y: Math.round(c.componentGeometry.y).toString(),
              width: Math.round(c.componentGeometry.width).toString(),
              height: Math.round(c.componentGeometry.height).toString(),
            },
          }));

          return new ApiRequest<{
            id: string;
          }>({
            method: "post",
            url: API_PREFIX.concat([
              { viewId: this.selectedViewId },
              "paste_components",
            ]),
            keyRequestStatusBy: components.map((c) => c.id),
            params: {
              components: APIComponents,
              newParentNodeId,
              ...visibilityParams,
            },
            optimistic: () => {
              for (const { id: tempId, position } of tempInserts) {
                this.pendingInsertedComponents[tempId] = {
                  tempId,
                  position,
                };
              }

              return () => {
                for (const { id } of tempInserts) {
                  delete this.pendingInsertedComponents[id];
                }
              };
            },
            onSuccess: () => {
              for (const { id } of tempInserts) {
                delete this.pendingInsertedComponents[id];
              }
            },
          });
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
          [
            {
              eventType: "ComponentCreated",
              callback: (data) => {
                // If the component that updated wasn't in this change set,
                // don't update
                if (data.changeSetId !== changeSetId) return;
                const { viewId, geometry } = { ...data.component.viewData };
                if (!viewId || !geometry)
                  throw new Error("Expected view geometry on new component");
                const view = this.viewsById[viewId];
                if (!view) return; // FIXME later when we have full WsEvents
                if (data.component.componentType === ComponentType.Component) {
                  const node = processRawComponent(
                    data.component,
                    componentsStore.rawComponentsById,
                  ) as DiagramNodeData;
                  geometry.height = node.height;
                  geometry.width = node.width;
                  view.components[data.component.id] = geometry as IRect;
                  for (const [key, loc] of Object.entries(
                    setSockets(node, geometry),
                  )) {
                    view.sockets[key] = loc;
                  }
                } else {
                  if (!geometry.width) geometry.width = 500;
                  if (!geometry.height) geometry.height = 500;
                  view.groups[data.component.id] = geometry as IRect;
                  const node = processRawComponent(
                    data.component,
                    componentsStore.rawComponentsById,
                  ) as DiagramGroupData;
                  for (const [key, loc] of Object.entries(
                    setSockets(node, geometry),
                  )) {
                    view.sockets[key] = loc;
                  }
                }
              },
            },
            {
              eventType: "ComponentUpgraded",
              callback: (data) => {
                // If the component that updated wasn't in this change set,
                // don't update
                if (data.changeSetId !== changeSetId) return;
                const node = processRawComponent(
                  data.component,
                  componentsStore.rawComponentsById,
                );
                // upgrades can change the sockets on a component
                // so we need to go through all the views this component is on
                // and re-set their socket data
                const viewIds = Object.keys(this.viewsById);
                viewIds.forEach((viewId) => {
                  const view = this.viewsById[viewId];
                  if (!view) return;
                  let geo = view.components[data.component.id];
                  if (!geo) geo = view.groups[data.component.id];
                  if (geo) {
                    for (const [key, loc] of Object.entries(
                      setSockets(node, geo),
                    )) {
                      view.sockets[key] = loc;
                    }
                  }
                });
              },
            },
            {
              eventType: "ComponentUpdated",
              callback: (data, metadata) => {
                // If the component that updated wasn't in this change set,
                // don't update
                if (metadata.change_set_id !== changeSetId) return;
                const { viewId, geometry } = { ...data.component.viewData };
                if (!viewId || !geometry)
                  // this is expected in many situations
                  return; // but will be populated on changing componentType

                const view = this.viewsById[viewId];
                if (!view) return; // FIXME later when we have full WsEvents

                delete view.components[data.component.id];
                delete view.groups[data.component.id];
                if (data.component.componentType === ComponentType.Component) {
                  const node = processRawComponent(
                    data.component,
                    componentsStore.rawComponentsById,
                  ) as DiagramNodeData;
                  geometry.height = node.height;
                  geometry.width = node.width;
                  view.components[data.component.id] = geometry as IRect;
                } else {
                  if (!geometry.width) geometry.width = 500;
                  if (!geometry.height) geometry.height = 500;
                  view.groups[data.component.id] = geometry as IRect;
                }
              },
            },

            {
              eventType: "ComponentDeleted",
              callback: (data) => {
                if (data.changeSetId !== changeSetId) return;
                // TODO "detach component from view"
                // this is "delete from all views"
                const viewIds = Object.keys(this.viewsById);
                viewIds.forEach((viewId) => {
                  const view = this.viewsById[viewId];
                  delete view?.components[data.componentId];
                  delete view?.groups[data.componentId];
                });
              },
            },

            {
              eventType: "SetComponentPosition",
              callback: ({ changeSetId, clientUlid, viewId, positions }) => {
                if (changeSetId !== changeSetsStore.selectedChangeSetId) return;
                if (clientUlid === this.diagramUlid) return;
                const _view = this.viewsById[viewId];
                if (!_view) return;
                // TODO: make sure to update the correct view based on ID

                for (const geo of positions) {
                  const component =
                    componentsStore.allComponentsById[geo.componentId];
                  if (component) {
                    let viewComponent;
                    if (component.def.isGroup) {
                      viewComponent = _view.groups[geo.componentId];
                      if (viewComponent && geo.height && geo.width) {
                        viewComponent.height = geo.height;
                        viewComponent.width = geo.width;
                      }
                    } else {
                      viewComponent = _view.components[geo.componentId];
                    }
                    if (viewComponent) {
                      viewComponent.x = geo.x;
                      viewComponent.y = geo.y;
                    }

                    for (const [key, loc] of Object.entries(
                      setSockets(component, geo),
                    )) {
                      _view.sockets[key] = loc;
                    }
                  }
                }
              },
            },
          ],
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
          actionUnsub();
          realtimeStore.unsubscribe(`${this.$id}-changeset`);
          realtimeStore.unsubscribe(`${this.$id}-workspace`);
        };
      },
    }),
  )();
};
