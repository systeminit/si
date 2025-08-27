import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { IRect, Vector2d } from "konva/lib/types";
import { useToast } from "vue-toastification";
import { IconNames } from "@si/vue-lib/design-system";
import { URLPattern } from "@si/vue-lib";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import {
  ApprovalRequirementDefinitionId,
  Components,
  Groups,
  Sockets,
  StringGeometry,
  View,
  ViewApprovalRequirementDefinition,
  ViewDescription,
  ViewId,
  ViewNodes,
} from "@/api/sdf/dal/views";
import {
  DiagramGroupData,
  DiagramNodeData,
  DiagramViewData,
  ElementHoverMeta,
  Size2D,
} from "@/components/ModelingDiagram/diagram_types";
import {
  ComponentId,
  Edge,
  EdgeId,
  RawComponent,
  ViewNodeGeometry,
} from "@/api/sdf/dal/component";
import {
  GROUP_BOTTOM_INTERNAL_PADDING,
  GROUP_INTERNAL_PADDING,
  NODE_WIDTH,
  SOCKET_SIZE,
} from "@/components/ModelingDiagram/diagram_constants";
import { DefaultMap } from "@/utils/defaultmap";
import { ComponentType, SchemaVariant } from "@/api/sdf/dal/schema";
import { nonNullable } from "@/utils/typescriptLinter";
import router from "@/router";
import handleStoreError from "./errors";

import {
  diagramUlid as clientUlid,
  useChangeSetsStore,
} from "./change_sets.store";
import {
  ComponentsAndEdges,
  processRawComponent,
  useComponentsStore,
} from "./components.store";
import { useRealtimeStore } from "./realtime/realtime.store";
import { useWorkspacesStore } from "./workspaces.store";
import { useRouterStore } from "./router.store";
import { useQualificationsStore } from "./qualifications.store";
import { useAuthStore, UserId } from "./auth.store";
import { useFeatureFlagsStore } from "./feature_flags.store";

function vectorAdd(v1: Vector2d, v2: Vector2d) {
  return {
    x: v1.x + v2.x,
    y: v1.y + v2.y,
  } as Vector2d;
}

const MAX_RETRIES = 5;

type PendingComponent = {
  tempId: string;
  position: Vector2d;
};

type RequestUlid = string;

type VectorWithRadius = Vector2d & { radius: number };

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

export const VIEW_DEFAULTS = {
  icon: "create" as IconNames,
  color: "#9d00ff",
  schemaName: "View",
};

export interface ComponentStats {
  components: number;
  resources: number;
  failed: number;
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
  const qualStore = useQualificationsStore();
  const routerStore = useRouterStore();
  const authStore = useAuthStore();
  const realtimeStore = useRealtimeStore();
  const toast = useToast();
  const featureFlagsStore = useFeatureFlagsStore();

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

  const APPROVAL_REQUIREMENTS_API_PREFIX = [
    "v2",
    "workspaces",
    { workspaceId },
    "change-sets",
    { changeSetId },
    "approval-requirement-definitions",
  ] as URLPattern;

  return addStoreHooks(
    workspaceId,
    changeSetId,
    defineStore(`ws${workspaceId || "NONE"}/cs${changeSetId || "NONE"}/views`, {
      state: () => ({
        activatedAndFetched: false,
        selectedViewId: null as ViewId | null,
        outlinerViewId: null as ViewId | null,
        recentViews: new UniqueStack() as UniqueStack<ViewId>,

        // every views data goes here
        viewsById: {} as Record<ViewId, View>,
        viewList: [] as ViewDescription[],
        requirementDefinitionsById: {} as Record<
          ApprovalRequirementDefinitionId,
          ViewApprovalRequirementDefinition
        >,

        /* *
         * these hold the data for everything on the diagram in the SELECTED view
         * as selectedView changes we repopulate all this data
         * this is pushing together `movedElementPositions` and `resizedElementSizes`
         * and can make `renderedGeometriesByComponentId` unnecessary
         */
        components: {} as Components,
        groups: {} as Groups,
        sockets: {} as Sockets,
        viewNodes: {} as ViewNodes,

        // size of components when dragged to the stage
        inflightElementSizes: {} as Record<RequestUlid, ComponentId[]>,
        // prevents run away retries, unknown what circumstances could lead to this, but protecting ourselves
        inflightRetryCounter: new DefaultMap<string, number>(() => 0),
        pendingInsertedComponents: {} as Record<string, PendingComponent>,

        // componentId we drag from outliner into the selectedView
        addComponentId: null as ComponentId | null,
        // viewId we drag from left panel into the selectedView
        addViewId: null as ViewId | null,

        // these are components, groups, or viewNodes
        selectedComponentIds: [] as ComponentId[],
        selectedEdgeId: null as EdgeId | null,
        selectedDisplayEdgeId: null as EdgeId | null,
        selectedComponentDetailsTab: null as string | null,
        selectedViewDetailsId: null as ViewId | null,
        hoveredComponentId: null as ComponentId | null,
        hoveredEdgeId: null as EdgeId | null,
        hoveredComponentMeta: null as ElementHoverMeta | null,
      }),
      getters: {
        // this could get expensive, might want to stop computing this
        viewStats() {
          const stats: Record<ViewId, ComponentStats> = {};
          Object.values(this.viewsById).forEach((view) => {
            const ids = Object.keys(view.components).concat(
              Object.keys(view.groups),
            );
            const stat = {
              components: ids.length,
              failed: 0,
              resources: 0,
            };
            ids.forEach((id) => {
              const c = componentsStore.allComponentsById[id];
              if (!c) return;
              if (c.def.hasResource) stat.resources++;
              const qual = qualStore.qualificationStatsByComponentId[id];
              if (qual?.failed) stat.failed++;
            });
            stats[view.id] = stat;
          });
          return stats;
        },
        hoveredComponent(): DiagramNodeData | DiagramGroupData | undefined {
          return componentsStore.allComponentsById[
            this.hoveredComponentId || 0
          ];
        },

        selectedComponentId: (state) => {
          return state.selectedComponentIds.length === 1
            ? state.selectedComponentIds[0]
            : null;
        },
        viewListCount: (state) => {
          return state.viewList.length;
        },
        selectedComponent():
          | DiagramNodeData
          | DiagramGroupData
          | DiagramViewData
          | undefined {
          return (
            componentsStore.allComponentsById[this.selectedComponentId || 0] ??
            this.viewNodes[this.selectedComponentId || 0]
          );
        },
        selectedComponents(): (
          | DiagramNodeData
          | DiagramGroupData
          | DiagramViewData
        )[] {
          return _.compact(
            _.map(
              this.selectedComponentIds,
              (id) =>
                componentsStore.allComponentsById[id] ?? this.viewNodes[id],
            ),
          );
        },
        selectedEdge(): Edge | undefined {
          return componentsStore.rawEdgesById[this.selectedEdgeId || 0];
        },
        selectedComponentsAndChildren(): (
          | DiagramNodeData
          | DiagramGroupData
          | DiagramViewData
        )[] {
          const selectedAndChildren: Record<
            string,
            DiagramNodeData | DiagramGroupData | DiagramViewData
          > = {};
          Object.values(componentsStore.allComponentsById).forEach(
            (component) => {
              this.selectedComponents?.forEach((el) => {
                if (component.def.ancestorIds?.includes(el.def.id)) {
                  selectedAndChildren[component.def.id] = component;
                }
              });
            },
          );
          this.selectedComponents?.forEach((el) => {
            selectedAndChildren[el.def.id] = el;
          });

          return Object.values(selectedAndChildren);
        },

        deletableSelectedComponents(): (DiagramNodeData | DiagramGroupData)[] {
          const components: (DiagramNodeData | DiagramGroupData)[] = [];
          this.selectedComponentsAndChildren.forEach((c) => {
            if (
              !(c instanceof DiagramViewData) &&
              "changeStatus" in c.def &&
              c.def.changeStatus !== "deleted"
            ) {
              components.push(c);
            }
          });
          return components;
        },
        restorableSelectedComponents(): (DiagramNodeData | DiagramGroupData)[] {
          const components: (DiagramNodeData | DiagramGroupData)[] = [];
          this.selectedComponentsAndChildren.forEach((c) => {
            if (
              !(c instanceof DiagramViewData) &&
              "changeStatus" in c.def &&
              c.def.changeStatus === "deleted"
            ) {
              components.push(c);
            }
          });
          return components;
        },
        erasableSelectedComponents(): (DiagramNodeData | DiagramGroupData)[] {
          const components: (DiagramNodeData | DiagramGroupData)[] = [];
          this.selectedComponentsAndChildren.forEach((c) => {
            if (
              !(c instanceof DiagramViewData) &&
              "changeStatus" in c.def &&
              c.def.changeStatus !== "deleted"
            ) {
              components.push(c);
            }
          });
          return components;
        },
        detailsTabSlugs: (state) => {
          const slug = state.selectedComponentDetailsTab;

          // root level tabs
          if (["resource", "management", "component"].includes(slug || "")) {
            return [slug, undefined];
          }

          // subtabs
          if (slug?.startsWith("management-")) return ["management", slug];
          if (slug?.startsWith("resource-")) return ["resource", slug];

          // all other subtabs (currently) are in the component tab
          return ["component", slug];
        },
        diagramIsEmpty(state): boolean {
          return (
            Object.keys(state.components).length === 0 &&
            Object.keys(state.groups).length === 0 &&
            Object.keys(state.viewNodes).length === 0
          );
        },

        defaultViewId: (state) => {
          const idx = state.viewList.findIndex((v) => v.isDefault);
          return idx !== -1 ? state.viewList[idx]?.id : state.viewList[0]?.id;
        },
        edges: (state) => {
          const subscriptions = featureFlagsStore.SIMPLE_SOCKET_UI
            ? _.values(componentsStore.diagramSubscriptionEdgesById)
            : [];

          const edges = Object.values(componentsStore.diagramEdgesById)
            .concat(subscriptions)
            .filter((e) => {
              const to = e.toNodeKey.substring(2);
              const from = e.fromNodeKey.substring(2);
              const componentIds = Object.keys(state.components).concat(
                Object.keys(state.groups),
              );
              return componentIds.includes(to) && componentIds.includes(from);
            });

          return edges;
        },
        selectedView: (state) => state.viewsById[state.selectedViewId || ""],
        outlinerView: (state) => state.viewsById[state.outlinerViewId || ""],
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
            ) {
              continue;
            }

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

        requirementDefintionsByViewId: (state) => {
          const out = {} as Record<ViewId, ViewApprovalRequirementDefinition[]>;
          for (const requirement of Object.values(
            state.requirementDefinitionsById,
          )) {
            if (!out[requirement.entityId]) {
              out[requirement.entityId] = [];
            }
            out[requirement.entityId]?.push(requirement);
          }
          return out;
        },

        viewNamesByComponentId: (state) => {
          const record = {} as Record<ComponentId, string[]>;
          const views = _.values(state.viewsById);

          for (const view of views) {
            for (const componentId of _.keys(view.components)) {
              record[componentId] ??= [];
              record[componentId]?.push(view.name);
            }
            for (const groupId of _.keys(view.groups)) {
              record[groupId] ??= [];
              record[groupId]?.push(view.name);
            }
          }

          return record;
        },
      },
      actions: {
        geoFrom(
          el: DiagramNodeData | DiagramGroupData | DiagramViewData,
        ): IRect | undefined {
          let geo: IRect | undefined;
          if (el.def.componentType === ComponentType.View) {
            const v = this.viewNodes[el.def.id];
            if (v) {
              geo = {
                x: v.def.x,
                y: v.def.y,
                width: v.def.width,
                height: v.def.height,
              };
            }
          } else {
            geo = el.def.isGroup
              ? this.groups[el.def.id]
              : this.components[el.def.id];
          }
          return geo;
        },
        setHoveredComponentId(id: ComponentId | null, meta?: ElementHoverMeta) {
          this.hoveredComponentId = id;
          this.hoveredComponentMeta = meta || null;
          this.hoveredEdgeId = null;
        },
        setHoveredEdgeId(id: ComponentId | null) {
          this.hoveredComponentId = null;
          this.hoveredEdgeId = id;
        },

        setComponentDetailsTab(tabSlug: string | null) {
          // we ignore the top level "component" and "actions" tabs
          // since we always need a child selected, and setting these
          // would overwrite the child being selected
          if (["component", "actions"].includes(tabSlug || "")) return;
          this.selectedComponentDetailsTab = tabSlug;

          this.syncSelectionIntoUrl();
        },
        syncSelectionIntoUrl() {
          let selectedIds: string[] = [];
          if (this.selectedEdgeId) {
            selectedIds = [`e_${this.selectedEdgeId}`];
          } else if (this.selectedComponentIds.length) {
            selectedIds = _.map(this.selectedComponentIds, (id) => `c_${id}`);
          }

          const newQueryObj = {
            ...(selectedIds.length && { s: selectedIds.join("|") }),
            ...(this.selectedComponentDetailsTab && {
              t: this.selectedComponentDetailsTab,
            }),
          };

          if (!_.isEqual(routerStore.currentRoute?.query, newQueryObj)) {
            routerStore.replace(changeSetId, {
              params: { ...routerStore.currentRoute?.params },
              query: newQueryObj,
            });
          }
        },
        syncUrlIntoSelection() {
          const ids = (
            (router.currentRoute.value.query?.s as string) || ""
          ).split("|");
          if (!ids.length) {
            this.selectedComponentIds = [];
            this.selectedEdgeId = null;
            this.selectedViewDetailsId = null;
          } else if (ids.length === 1 && ids[0]?.startsWith("e_")) {
            this.selectedComponentIds = [];
            this.selectedEdgeId = ids[0].substring(2);
            this.selectedViewDetailsId = null;
          } else {
            this.selectedComponentIds = ids.map((id) => id.substring(2));
            this.selectedEdgeId = null;
            this.selectedViewDetailsId = null;
          }

          const tabSlug =
            (router.currentRoute.value.query?.t as string) || null;
          if (this.selectedComponentIds.length === 1) {
            this.selectedComponentDetailsTab = tabSlug;
          } else {
            this.selectedComponentDetailsTab = null;
          }
        },
        setSelectedViewDetails(id: ViewId) {
          this.selectedViewDetailsId = id;
          this.selectedComponentIds = [];
          this.selectedEdgeId = null;
          this.selectedDisplayEdgeId = null;
        },
        setSelectedEdgeId(
          selection: EdgeId | null,
          displayEdgeId?: EdgeId | null,
        ) {
          // clear component selection
          this.selectedViewDetailsId = null;
          this.selectedComponentIds = [];
          this.selectedEdgeId = selection;
          this.selectedComponentDetailsTab = null;
          if (displayEdgeId) this.selectedDisplayEdgeId = displayEdgeId;
          this.syncSelectionIntoUrl();
        },
        setSelectedComponentId(
          selection: ComponentId | ComponentId[] | null,
          opts?: { toggle?: boolean; detailsTab?: string },
        ) {
          const key = `${changeSetId}_selected_component`;
          this.selectedViewDetailsId = null;
          this.selectedEdgeId = null;
          this.selectedDisplayEdgeId = null;
          if (!selection || !selection.length) {
            this.selectedComponentIds = [];
            // forget which details tab is active when selection is cleared
            this.selectedComponentDetailsTab = null;
            if (router.currentRoute.value.name === "workspace-compose") {
              window.localStorage.removeItem(key);
            }
          } else {
            const validSelectionArray = _.reject(
              _.isArray(selection) ? selection : [selection],
              (id) =>
                !Object.keys(componentsStore.allComponentsById).includes(id) &&
                !Object.keys(this.viewNodes).includes(id),
            );
            if (opts?.toggle) {
              this.selectedComponentIds = _.xor(
                this.selectedComponentIds,
                validSelectionArray,
              );
            } else {
              this.selectedComponentIds = validSelectionArray;
            }
          }
          if (opts?.detailsTab) {
            this.selectedComponentDetailsTab = opts.detailsTab;
          }
          if (this.selectedComponentIds.length === 1) {
            const _id = this.selectedComponentIds[0];
            if (_id) window.localStorage.setItem(key, _id);
          }

          this.syncSelectionIntoUrl();
        },
        setOutlinerView(id: ViewId) {
          this.outlinerViewId = id;
          this._enforceSelectedComponents();
        },
        // cannot select components that are not in either selected view
        _enforceSelectedComponents() {
          const v1 = this.viewsById[this.outlinerViewId || ""];
          const v2 = this.viewsById[this.selectedViewId || ""];
          const ids: ComponentId[] = [];
          if (v1) {
            ids.push(...Object.keys(v1.components));
            ids.push(...Object.keys(v1.groups));
          }
          if (v2) {
            ids.push(...Object.keys(v2.components));
            ids.push(...Object.keys(v2.groups));
          }
          const valid = this.selectedComponentIds.filter((cId) =>
            ids.includes(cId),
          );
          this.selectedComponentIds = valid;
          this.syncSelectionIntoUrl();
        },
        viewAssignment(view: View) {
          /* *
           * i think i want to set these as in-memory references
           * that way i don't have to do two writes for incoming WsEvents
           * or two writes for user actions
           *
           * this does mean that `draggedElementsPositionsPreDrag` and
           * `resizedElementSizesPreResize` need to be populated
           * but those could just be a `structuredClone` of this data
           */
          this.components = view.components;
          this.groups = view.groups;
          this.viewNodes = view.viewNodes;
          // derive the socket position from the component position
          // to begin, and then adjust it via delta when things move
          this.sockets = view.sockets;
        },
        clearSelectedView() {
          this.selectedViewId = null;
          this.sockets = {};
          this.components = {};
          this.groups = {};
          this.viewNodes = {};
        },
        selectView(id: ViewId, navigable = true) {
          const view = this.viewsById[id];
          if (view) {
            if (navigable) {
              const route = router.currentRoute;
              const params = {
                ...route.value.params,
                viewId: id,
              };
              routerStore.push(changeSetId, {
                name: "workspace-compose-view",
                params,
              });
            }

            // move the currently selected view to the top of the
            if (this.selectedViewId) {
              this.pushRecentView(this.selectedViewId);
            }

            this.selectedViewId = id;
            this.outlinerViewId = id;
            this._enforceSelectedComponents();
            this.viewAssignment(view);
            this.setGroupZIndex();
          } else {
            this.FETCH_VIEW(id);
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
            onSuccess: async (views) => {
              this.viewList = views;
              this.SORT_LIST_VIEWS();
            },
          });
        },
        async FETCH_COMPLETE_DATA() {
          await componentsStore.FETCH_ALL_COMPONENTS();
          const ids = this.viewList.map((v) => v.id);
          await Promise.all([...ids.map((id) => this.FETCH_VIEW_GEOMETRY(id))]);
        },
        SORT_LIST_VIEWS() {
          this.viewList = this.viewList.sort((a, b) => {
            if (a.isDefault) return -1;
            if (b.isDefault) return 1;
            return a.name.localeCompare(b.name);
          });
        },
        async FETCH_VIEW_GEOMETRY(viewId: ViewId) {
          // requires all components to be in place!
          return new ApiRequest<{
            viewId: ViewId;
            name: string;
            components: Record<ComponentId, Vector2d & Partial<Size2D>>;
            views: Record<ViewId, Vector2d & Size2D>;
          }>({
            method: "get",
            url: API_PREFIX.concat([{ viewId }, "get_geometry"]),
            onSuccess: (response) => {
              const view: View = {
                id: response.viewId,
                name: response.name,
                components: {},
                groups: {},
                sockets: {},
                viewNodes: {},
              };
              this.viewsById[response.viewId] = view;

              Object.entries(response.views).forEach(([viewId, geo]) => {
                const v = this.viewList.find((_v) => _v.id === viewId);
                if (!v) return;

                view.viewNodes[v.id] = new DiagramViewData({
                  ...VIEW_DEFAULTS,
                  ...geo,
                  // if geo has properties like "id" we don't want them to overwrite view props
                  // so we spread v last
                  ...v,
                  componentType: ComponentType.View,
                });
              });

              Object.entries(response.components).forEach(
                ([componentId, geo]) => {
                  const node = componentsStore.allComponentsById[componentId];
                  if (!node) return;
                  let geometry: IRect;
                  if ("width" in node) {
                    geo.width = node.width;
                    geo.height = node.height;
                    geometry = { ...(geo as IRect) };
                    view.components[componentId] = geometry;
                  } else {
                    geometry = { ...(geo as IRect) };
                    if (!geometry.width) geometry.width = 500;
                    if (!geometry.height) geometry.height = 500;
                    view.groups[componentId] = {
                      // this one is actually an IRect
                      ...geometry,
                      size: geometry.width * geometry.height,
                      zIndex: 0,
                    };
                  }
                  for (const [key, loc] of Object.entries(
                    setSockets(node, geometry),
                  )) {
                    view.sockets[key] = loc;
                  }
                },
              );
              if (this.selectedViewId === view.id) this.viewAssignment(view);
              this.setGroupZIndex();
            },
          });
        },
        setAddComponentId(id: ComponentId) {
          this.addComponentId = id;
          this.setSelectedComponentId(null);
        },
        cancelAdd() {
          this.addComponentId = null;
        },
        removeSelectedViewComponentFromCurrentView() {
          if (this.selectedViewId) {
            const componentIds = this.selectedComponents
              .filter((c) => c.def.componentType !== ComponentType.View)
              .map((c) => c.def.id);
            if (componentIds.length > 0) {
              this.REMOVE_FROM(this.selectedViewId, componentIds);
            }
            const viewIds = this.selectedComponents
              .filter((c) => c.def.componentType === ComponentType.View)
              .map((c) => c.def.id);
            if (viewIds.length > 0) {
              this.REMOVE_VIEW_FROM(this.selectedViewId, viewIds);
            }
          }
        },
        async CREATE_VIEW(name: string) {
          return new ApiRequest<ViewDescription>({
            method: "post",
            url: API_PREFIX,
            params: { name, clientUlid },
          });
        },
        async UPDATE_VIEW_NAME(view_id: ViewId, name: string) {
          return new ApiRequest<null>({
            method: "put",
            url: API_PREFIX.concat([view_id]),
            params: { name, clientUlid },
            onSuccess: () => {
              componentsStore.eventBus.emit("renameView", view_id);
            },
          });
        },

        async FETCH_VIEW(viewId?: ViewId) {
          let url = API_PREFIX.concat(["default", "get_diagram"]);
          if (viewId) url = API_PREFIX.concat([{ viewId }, "get_diagram"]);

          return new ApiRequest<{
            view: ViewDescription;
            diagram: ComponentsAndEdges & {
              views: ViewNodeGeometry[];
            };
          }>({
            url,
            params: {
              ...visibilityParams,
            },
            onFail: () => {
              // getting the view id from URL can fail if you've moved changesets
              const route = useRouterStore().currentRoute;
              if (
                !this.selectedViewId &&
                route?.name === "workspace-compose-view"
              ) {
                const params = { ...route.params };
                if ("viewId" in params) delete params.viewId;
                routerStore.push(changeSetId, {
                  name: "workspace-compose",
                  params,
                });
              }
            },
            onSuccess: (response) => {
              componentsStore.SET_COMPONENTS_FROM_VIEW(response.diagram, {
                representsAllComponents: false,
              });

              // remove invalid component IDs from the selection
              const validComponentIds = _.intersection(
                this.selectedComponentIds,
                _.keys(componentsStore.rawComponentsById),
              );
              this.setSelectedComponentId(validComponentIds);

              const components: RawComponent[] = [];
              const groups: RawComponent[] = [];
              for (const component of response.diagram.components) {
                if (component) {
                  if (component.componentType === ComponentType.Component) {
                    components.push(component);
                  } else groups.push(component);
                }
              }
              const views: (ViewDescription & IRect)[] = [];
              response.diagram.views.forEach((v) => {
                views.push({ ...v.view, ...v.geometry });
              });
              this.SET_COMPONENTS_FROM_VIEW(response.view, {
                components,
                groups,
                views,
              });
              this.selectView(response.view.id, false);
              this.setGroupZIndex();
            },
          });
        },
        SET_COMPONENTS_FROM_VIEW(
          view: ViewDescription,
          response: {
            components: RawComponent[];
            groups: RawComponent[];
            views: (ViewDescription & IRect)[];
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
              size: geometry.width * geometry.height,
              zIndex: 0,
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
          const viewNodes: ViewNodes = {};
          response.views.forEach((v) => {
            viewNodes[v.id] = new DiagramViewData({
              ...v,
              ...VIEW_DEFAULTS,
              componentType: ComponentType.View,
            });
          });
          this.viewsById[view.id] = {
            id: view.id,
            name: view.name,
            components,
            groups,
            sockets,
            viewNodes,
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
          components: (DiagramGroupData | DiagramNodeData)[],
          positionDelta: Vector2d,
          opts: { writeToChangeSet?: boolean; broadcastToClients?: boolean },
        ) {
          if (positionDelta.x !== 0 || positionDelta.y !== 0) {
            components.forEach((c) => {
              const orig = c.def.isGroup
                ? this.groups[c.def.id]
                : this.components[c.def.id];
              if (!orig) return;

              const newPos = vectorAdd({ ...orig }, positionDelta);
              orig.x = newPos.x;
              orig.y = newPos.y;

              if (featureFlagsStore.SIMPLE_SOCKET_UI) {
                const width = c.def.isGroup
                  ? this.groups[c.def.id]?.width
                  : this.components[c.def.id]?.width;
                if (!width) return;

                const displaySockets = [
                  ...c.layoutLeftSockets(width).sockets,
                  ...c.layoutRightSockets(width).sockets,
                ];

                new Set(displaySockets.map((s) => s.uniqueKey)).forEach(
                  (uniqueKey) => {
                    const geo = this.sockets[uniqueKey];
                    if (!geo) return;
                    geo.center.x += positionDelta.x;
                    geo.center.y += positionDelta.y;
                  },
                );
              } else {
                c.sockets.forEach((s) => {
                  const geo = this.sockets[s.uniqueKey];
                  if (!geo) return;
                  geo.center.x += positionDelta.x;
                  geo.center.y += positionDelta.y;
                });
              }
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
              if (geo) {
                payload[c.def.id] = {
                  x: Math.round(geo.x).toString(),
                  y: Math.round(geo.y).toString(),
                  width: Math.round(geo.width).toString(),
                  height: Math.round(geo.height).toString(),
                };
              }
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
        async MOVE_VIEWS(
          components: DiagramViewData[],
          positionDelta: Vector2d,
          opts: { writeToChangeSet?: boolean; broadcastToClients?: boolean },
        ) {
          if (positionDelta.x !== 0 || positionDelta.y !== 0) {
            components.forEach((c) => {
              const v = this.viewNodes[c.def.id];
              if (!v) return;

              const newPos = vectorAdd(
                { x: v.def.x, y: v.def.y },
                positionDelta,
              );
              v.def.x = newPos.x;
              v.def.y = newPos.y;
            });
          }
          if (!opts.broadcastToClients && !opts.writeToChangeSet) return;

          if (opts.broadcastToClients && changeSetsStore.selectedChangeSetId) {
            const payload: Record<ComponentId, IRect> = {};
            components.forEach((c) => {
              const v = this.viewNodes[c.def.id];
              if (!v) return;
              payload[c.def.id] = {
                x: Math.round(v.def.x),
                y: Math.round(v.def.y),
                width: Math.round(v.def.width),
                height: Math.round(v.def.height),
              };
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
              const geo = this.viewNodes[c.def.id]?.def;
              if (geo) {
                payload[c.def.id] = {
                  x: Math.round(geo.x).toString(),
                  y: Math.round(geo.y).toString(),
                  width: Math.round(geo.width).toString(),
                  height: Math.round(geo.height).toString(),
                };
              }
            });
            return new ApiRequest<{ requestUlid: RequestUlid }>({
              method: "put",
              url: API_PREFIX.concat([
                { viewId: this.selectedViewId },
                "view_object/set_geometry",
              ]),
              params: {
                clientUlid,
                dataByViewId: payload,
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
                    .map((componentId) => this.viewNodes[componentId])
                    .filter(nonNullable);
                  this.MOVE_VIEWS(
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
          component: DiagramGroupData,
          geometry: IRect,
          opts: { writeToChangeSet?: boolean; broadcastToClients?: boolean },
        ) {
          geometry.x = Math.round(geometry.x);
          geometry.y = Math.round(geometry.y);
          geometry.width = Math.round(geometry.width);
          geometry.height = Math.round(geometry.height);

          this.groups[component.def.id] = {
            ...geometry,
            size: geometry.width * geometry.height,
            zIndex: 0,
          };
          this.setGroupZIndex();

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
          componentIds: ComponentId[],
          newParentId: ComponentId | null,
        ) {
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
                else {
                  component.parentId = undefined;
                }
                componentsStore.processAndStoreRawComponent(componentId, {});
              });
              // if we change to no parent, we have to follow up and re-process
              Object.values(oldParentIds)
                .filter(nonNullable)
                .forEach((parentId) => {
                  componentsStore.processAndStoreRawComponent(parentId, {});
                });
            },
            onFail: () => {
              componentIds.forEach((componentId) => {
                const component =
                  componentsStore.rawComponentsById[componentId];
                if (!component) return;
                component.parentId = oldParentIds[componentId];
                componentsStore.processAndStoreRawComponent(componentId, {});
                if (component.parentId) {
                  componentsStore.processAndStoreRawComponent(
                    component.parentId,
                    {},
                  );
                }
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
          if (changeSetsStore.creatingChangeSet) {
            throw new Error("race, wait until the change set is created");
          }
          if (changeSetId === changeSetsStore.headChangeSetId) {
            changeSetsStore.creatingChangeSet = true;
          }

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
          if (changeSetsStore.creatingChangeSet) {
            throw new Error("race, wait until the change set is created");
          }
          if (changeSetId === changeSetsStore.headChangeSetId) {
            changeSetsStore.creatingChangeSet = true;
          }

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

        async REMOVE_FROM(viewId: ViewId, componentIds: ComponentId[]) {
          return new ApiRequest({
            method: "delete",
            url: API_PREFIX.concat([{ viewId }, "erase_components"]),
            params: {
              clientUlid,
              componentIds,
              ...visibilityParams,
            },
            onFail: (err) => {
              if (err.response.status === 403) {
                toast(
                  "Error: Could not remove component(s) from this View. They do not exist in any other view, and would be orphaned.",
                );
              }
            },
          });
        },

        async ADD_TO(
          sourceViewId: ViewId,
          components: Record<ComponentId, IRect>,
          destinationViewId: ViewId,
          removeFromOriginalView = false,
        ) {
          const stringGeometries: Record<ComponentId, StringGeometry> = {};
          Object.entries(components).forEach(([componentId, geo]) => {
            stringGeometries[componentId] = {
              x: Math.round(geo.x).toString(),
              y: Math.round(geo.y).toString(),
              width: Math.round(geo.width).toString(),
              height: Math.round(geo.height).toString(),
            };
          });
          return new ApiRequest({
            method: "post",
            url: "diagram/add_components_to_view",
            params: {
              clientUlid,
              sourceViewId,
              destinationViewId,
              geometriesByComponentId: stringGeometries,
              removeFromOriginalView,
              ...visibilityParams,
            },
            onFail: (err) => {
              if (err.response.status === 422) {
                toast(
                  "Error: One or more of the selected components already exists in this view",
                );
              }
            },
          });
        },
        async ADD_VIEW_TO(
          viewId: ViewId,
          viewIdToHexagon: ViewId,
          geo: VectorWithRadius,
        ) {
          const sGeo = {
            x: Math.round(geo.x).toString(),
            y: Math.round(geo.y).toString(),
            radius: Math.round(geo.radius).toString(),
          };
          return new ApiRequest({
            method: "post",
            url: API_PREFIX.concat([{ viewId }, "view_object"]),
            params: {
              viewObjectId: viewIdToHexagon,
              ...sGeo,
            },
          });
        },
        async REMOVE_VIEW_FROM(viewId: ViewId, viewIdsToHexagon: ViewId[]) {
          return new ApiRequest({
            method: "delete",
            url: API_PREFIX.concat([{ viewId }, "view_object"]),
            params: {
              viewIds: viewIdsToHexagon,
            },
          });
        },
        async DELETE_VIEW(viewId: ViewId) {
          return new ApiRequest({
            method: "delete",
            url: API_PREFIX.concat([{ viewId }]),
            params: {
              viewId,
            },
          });
        },
        async CONVERT_TO_VIEW(
          sourceViewId: ViewId,
          componentId: ComponentId,
          containedComponentIds: ComponentId[],
        ) {
          const placeViewAt = this.groups[componentId];
          const viewToHexagonGeo = {
            x: placeViewAt?.x.toString(),
            y: placeViewAt?.y.toString(),
            radius: "250",
          };
          const stringGeometries: Record<ComponentId, StringGeometry> = {};
          containedComponentIds.forEach((c) => {
            const geo = this.components[c];
            stringGeometries[c] = {
              x: Math.round(geo?.x || 0).toString(),
              y: Math.round(geo?.y || 0).toString(),
              width: Math.round(geo?.width || 0).toString(),
              height: Math.round(geo?.height || 0).toString(),
            };
          });
          return new ApiRequest({
            method: "post",
            url: API_PREFIX.concat(["convert_to_view"]),
            params: {
              componentId,
              sourceViewId,
              containedComponentIds: stringGeometries,
              placeViewAt: viewToHexagonGeo,
            },
            optimistic: () => {
              this.setSelectedComponentId(null);
              this.syncSelectionIntoUrl();
            },
          });
        },
        async CREATE_VIEW_AND_MOVE(
          name: string,
          sourceViewId: ViewId,
          components: Record<ComponentId, IRect>,
        ) {
          let sumX = 0;
          let sumY = 0;
          const stringGeometries: Record<ComponentId, StringGeometry> = {};
          Object.entries(components).forEach(([componentId, geo]) => {
            // sum the center coordinates separately
            sumX += geo.x;
            sumY += geo.y + geo.height / 2;
            stringGeometries[componentId] = {
              x: Math.round(geo.x).toString(),
              y: Math.round(geo.y).toString(),
              width: Math.round(geo.width).toString(),
              height: Math.round(geo.height).toString(),
            };
          });
          const viewToHexagonGeo = {
            x: Math.round(
              sumX / Object.keys(stringGeometries).length,
            ).toString(),
            y: Math.round(
              sumY / Object.keys(stringGeometries).length,
            ).toString(),
            radius: "250",
          };
          return new ApiRequest({
            method: "post",
            url: API_PREFIX.concat(["create_and_move"]),
            params: {
              name,
              sourceViewId,
              geometriesByComponentId: stringGeometries,
              removeFromOriginalView: true,
              placeViewAt: viewToHexagonGeo,
            },
          });
        },

        // view approval requirement endpoints
        async CREATE_VIEW_APPROVAL_REQUIREMENT(viewId: ViewId, userId: UserId) {
          return new ApiRequest({
            method: "put",
            url: APPROVAL_REQUIREMENTS_API_PREFIX,
            params: {
              entityId: viewId,
              users: [userId],
            },
          });
        },
        async REMOVE_VIEW_APPROVAL_REQUIREMENT(
          definitionId: ApprovalRequirementDefinitionId,
        ) {
          return new ApiRequest({
            method: "delete",
            url: APPROVAL_REQUIREMENTS_API_PREFIX.concat([definitionId]),
          });
        },
        async LIST_VIEW_APPROVAL_REQUIREMENTS(viewId: ViewId) {
          return new ApiRequest({
            method: "get",
            url: APPROVAL_REQUIREMENTS_API_PREFIX.concat(["entity", viewId]),
            onSuccess: (response: ViewApprovalRequirementDefinition[]) => {
              this.requirementDefinitionsById = {} as Record<
                ApprovalRequirementDefinitionId,
                ViewApprovalRequirementDefinition
              >;
              for (const def of response) {
                this.requirementDefinitionsById[def.id] = def;
              }
            },
          });
        },
        async ADD_INDIVIDUAL_APPROVER_TO_REQUIREMENT(
          definitionId: ApprovalRequirementDefinitionId,
          userId: UserId,
        ) {
          return new ApiRequest({
            method: "put",
            url: APPROVAL_REQUIREMENTS_API_PREFIX.concat([
              definitionId,
              "individual-approver",
              userId,
            ]),
          });
        },
        async REMOVE_INDIVIDUAL_APPROVER_FROM_REQUIREMENT(
          definitionId: ApprovalRequirementDefinitionId,
          userId: UserId,
        ) {
          return new ApiRequest({
            method: "delete",
            url: APPROVAL_REQUIREMENTS_API_PREFIX.concat([
              definitionId,
              "individual-approver",
              userId,
            ]),
          });
        },

        setGroupZIndex() {
          const groupSizes: {
            id: string;
            size: number;
          }[] = Object.entries(this.groups).map(([id, g]) => {
            return { id, size: g.size };
          });
          groupSizes.sort((a, b) => b.size - a.size);
          for (const [idx, groupSize] of groupSizes.entries()) {
            // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
            const g = this.groups[groupSize.id]!;
            g.zIndex = idx;
          }
        },
        registerRequestsBegin(requestUlid: string, actionName: string) {
          realtimeStore.inflightRequests.set(requestUlid, actionName);
        },
        registerRequestsEnd(requestUlid: string) {
          realtimeStore.inflightRequests.delete(requestUlid);
        },
      },
      async onActivated() {
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
                if (!viewId || !geometry) {
                  throw new Error("Expected view geometry on new component");
                }
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
                  view.groups[data.component.id] = {
                    ...(geometry as IRect),
                    size: geometry.width * geometry.height,
                    zIndex: 0,
                  };
                  this.setGroupZIndex();
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
              eventType: "ResourceRefreshed",
              callback: (data) => {
                // If the component that updated wasn't in this change set,
                // don't update
                if (data.changeSetId !== changeSetId) return;
              },
            },
            {
              eventType: "ComponentUpgraded",
              callback: (data, metadata) => {
                // If the component that updated wasn't in this change set,
                // don't update
                if (data.changeSetId !== changeSetId) return;

                const oldId = data.originalComponentId;
                delete componentsStore.rawComponentsById[oldId];
                delete componentsStore.allComponentsById[oldId];
                delete componentsStore.nodesById[oldId];
                delete componentsStore.groupsById[oldId];
                componentsStore.rawComponentsById[data.component.id] =
                  data.component;
                componentsStore.processAndStoreRawComponent(
                  data.component.id,
                  {},
                ); // now the component exists in the component store

                const node =
                  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
                  componentsStore.allComponentsById[data.component.id]!;

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
                    if ("height" in node) {
                      // this covers components
                      geo.height = node.height;
                      geo.width = node.width;
                      // note: if a person added a hundred sockets to a component
                      // and that component was a frame, it would not resize itself
                      // and the sockets would appear outside the frame
                    }
                    for (const [key, loc] of Object.entries(
                      setSockets(node, geo),
                    )) {
                      view.sockets[key] = loc;
                    }
                  }
                });

                if (
                  metadata.actor !== "System" &&
                  metadata.actor.User === authStore.userPk
                ) {
                  this.setSelectedComponentId(data.component.id);
                }
              },
            },
            {
              eventType: "ComponentUpdated",
              callback: (data, metadata) => {
                // If the component that updated wasn't in this change set,
                // don't update
                if (metadata.change_set_id !== changeSetId) return;
                const { viewId, geometry } = { ...data.component.viewData };

                // changed component type means book-keeping for all views
                // PSA: currently SDF does not change any geometry when you change component types
                // if it starts to change geometry, it must return new geometry for every view
                // the structure doesn't currently support that
                Object.values(this.viewsById).forEach((view) => {
                  const groupGeo = view.groups[data.component.id];
                  const componentGeo = view.components[data.component.id];
                  const thisGeo = groupGeo ?? componentGeo;
                  // I don't exist in this view, and I am not being added to this view, return
                  if (viewId && viewId !== view.id) {
                    return;
                  }
                  const finalGeo = geometry ?? thisGeo;
                  if (!finalGeo) return;

                  delete view.components[data.component.id];
                  delete view.groups[data.component.id];
                  if (
                    data.component.componentType === ComponentType.Component
                  ) {
                    const node = processRawComponent(
                      data.component,
                      componentsStore.rawComponentsById,
                    ) as DiagramNodeData;
                    finalGeo.height = node.height;
                    finalGeo.width = node.width;
                    view.components[data.component.id] = finalGeo as IRect;
                    for (const [key, loc] of Object.entries(
                      setSockets(node, finalGeo),
                    )) {
                      view.sockets[key] = loc;
                    }
                  } else {
                    if (!finalGeo.width) finalGeo.width = 500;
                    if (!finalGeo.height) finalGeo.height = 500;
                    view.groups[data.component.id] = {
                      ...(finalGeo as IRect),
                      size: finalGeo.width * finalGeo.height,
                      zIndex: 0,
                    };
                    const node = processRawComponent(
                      data.component,
                      componentsStore.rawComponentsById,
                    ) as DiagramGroupData;
                    for (const [key, loc] of Object.entries(
                      setSockets(node, finalGeo),
                    )) {
                      view.sockets[key] = loc;
                    }
                  }
                });
                this.setGroupZIndex();
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
                this.setGroupZIndex();

                // remove invalid component IDs from the selection
                const validComponentIds = _.intersection(
                  this.selectedComponentIds,
                  _.keys(componentsStore.rawComponentsById),
                );
                this.setSelectedComponentId(validComponentIds);
              },
            },

            {
              eventType: "SetComponentPosition",
              callback: ({
                changeSetId,
                clientUlid: _clientUlid,
                viewId,
                positions,
              }) => {
                if (changeSetId !== changeSetsStore.selectedChangeSetId) return;
                if (clientUlid === _clientUlid) return;
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
                  } else {
                    const node = _view.viewNodes[geo.componentId];
                    if (node) {
                      node.def.width = geo.width;
                      node.def.height = geo.height;
                      node.def.x = geo.x;
                      node.def.y = geo.y;
                    }
                  }
                }
              },
            },

            {
              eventType: "ViewDeleted",
              callback: ({ viewId }, metadata) => {
                if (metadata.change_set_id !== changeSetId) return;
                const idx = this.viewList.findIndex((v) => v.id === viewId);
                if (idx !== -1) this.viewList.splice(idx, 1);
                delete this.viewsById[viewId];
                this.SORT_LIST_VIEWS();
                Object.values(this.viewsById).forEach((view) => {
                  delete view.viewNodes[viewId];
                });
                const route = router.currentRoute;
                if (route.value.params.viewId === viewId) {
                  const defaultView = this.viewList.find((v) => v.isDefault);
                  if (defaultView) this.selectView(defaultView.id);
                  else {
                    const v = this.viewList[0];
                    if (v) this.selectView(v.id);
                    else {
                      router.push({
                        name: "workspace-single",
                        params: { workspacePk: workspaceId },
                      });
                    }
                  }
                }
              },
            },

            {
              eventType: "ViewUpdated",
              callback: ({ view }, metadata) => {
                if (metadata.change_set_id !== changeSetId) return;
                const idx = this.viewList.findIndex((v) => v.id === view.id);
                if (idx !== -1) this.viewList.splice(idx, 1, view);
                else {
                  this.viewList.push(view);
                }
                // right now the name is the only thing you can update
                const v = this.viewsById[view.id];
                if (v) v.name = view.name;
                const _v = this.viewNodes[view.id];
                if (_v) _v.def.name = view.name;

                this.SORT_LIST_VIEWS();
              },
            },

            {
              eventType: "ViewCreated",
              callback: async ({ view }, metadata) => {
                if (metadata.change_set_id !== changeSetId) return;
                const idx = this.viewList.findIndex((v) => v.id === view.id);
                if (idx !== -1) this.viewList.splice(idx, 1, view);
                else {
                  this.viewList.push(view);
                }
                this.SORT_LIST_VIEWS();
                await this.FETCH_VIEW_GEOMETRY(view.id);
                const actionWhichCreatedView =
                  realtimeStore.inflightRequests.get(metadata.request_ulid);
                if (
                  metadata.actor !== "System" &&
                  metadata.actor.User === authStore.userPk &&
                  actionWhichCreatedView !== "RUN_MGMT_PROTOTYPE"
                ) {
                  this.selectView(view.id);
                }
              },
            },
            {
              eventType: "ViewObjectCreated",
              callback: (payload, metadata) => {
                if (metadata.change_set_id !== changeSetId) return;
                const view = this.viewsById[payload.viewId];
                const v = this.viewList.find(
                  (_v) => _v.id === payload.viewObjectId,
                );
                if (v) {
                  const node = new DiagramViewData({
                    ...v,
                    ...payload.geometry,
                    ...VIEW_DEFAULTS,
                    componentType: ComponentType.View,
                  });
                  if (view) view.viewNodes[payload.viewObjectId] = node;
                }
              },
            },
            {
              eventType: "ViewObjectRemoved",
              callback: (payload, metadata) => {
                if (metadata.change_set_id !== changeSetId) return;
                const view = this.viewsById[payload.viewId];
                if (view) delete view.viewNodes[payload.viewObjectId];
              },
            },

            {
              eventType: "ViewComponentsUpdate",
              callback: (payload, metadata) => {
                if (metadata.change_set_id !== changeSetId) return;

                Object.entries(payload.updatesByView).forEach(
                  ([viewId, { added, removed }]) => {
                    const view = this.viewsById[viewId];
                    if (!view) return;
                    Object.entries(added).forEach(([componentId, geo]) => {
                      const c = componentsStore.allComponentsById[componentId];
                      if (!c) return;
                      if (c.def.isGroup) {
                        view.groups[componentId] = {
                          ...geo,
                          size: geo.height * geo.width,
                          zIndex: 0,
                        };
                      } else view.components[componentId] = geo;
                      if (geo) {
                        for (const [key, loc] of Object.entries(
                          setSockets(c, geo),
                        )) {
                          view.sockets[key] = loc;
                        }
                      }
                    });
                    removed.forEach((r) => {
                      delete view.components[r];
                      delete view.groups[r];
                    });
                  },
                );

                this.setGroupZIndex();
              },
            },

            // events for approval requirements
            {
              eventType: "ApprovalRequirementAddIndividualApprover",
              callback: (payload) => {
                const viewId =
                  this.requirementDefinitionsById[
                    payload.approvalRequirementDefinitionId
                  ]?.entityId;
                if (viewId) {
                  this.LIST_VIEW_APPROVAL_REQUIREMENTS(viewId);
                }
              },
            },
            {
              eventType: "ApprovalRequirementDefinitionCreated",
              callback: (payload) => {
                const viewId = payload.entityId;
                this.LIST_VIEW_APPROVAL_REQUIREMENTS(viewId);
              },
            },
            {
              eventType: "ApprovalRequirementDefinitionRemoved",
              callback: (payload) => {
                const viewId =
                  this.requirementDefinitionsById[
                    payload.approvalRequirementDefinitionId
                  ]?.entityId;
                if (viewId) {
                  this.LIST_VIEW_APPROVAL_REQUIREMENTS(viewId);
                }
              },
            },
            {
              eventType: "ApprovalRequirementRemoveIndividualApprover",
              callback: (payload) => {
                const viewId =
                  this.requirementDefinitionsById[
                    payload.approvalRequirementDefinitionId
                  ]?.entityId;
                if (viewId) {
                  this.LIST_VIEW_APPROVAL_REQUIREMENTS(viewId);
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
              callback: async (data) => {
                // If the applied change set has rebased into this change set,
                // then refetch (i.e. there might be updates!)
                if (data.toRebaseChangeSetId === changeSetId) {
                  this.FETCH_COMPLETE_DATA();
                }
              },
            },
          ],
        );

        const actionUnsub = this.$onAction(handleStoreError);

        return () => {
          // clear selection without triggering url stuff
          this.selectedComponentIds = [];
          this.selectedViewDetailsId = null;
          this.selectedEdgeId = null;
          this.selectedDisplayEdgeId = null;
          actionUnsub();
          realtimeStore.unsubscribe(`${this.$id}-changeset`);
          realtimeStore.unsubscribe(`${this.$id}-workspace`);
        };
      },
    }),
  )();
};
