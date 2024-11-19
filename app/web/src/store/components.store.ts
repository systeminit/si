import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { Vector2d } from "konva/lib/types";
import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { IconNames } from "@si/vue-lib/design-system";
import { useToast } from "vue-toastification";

import mitt from "mitt";
import { watch } from "vue";
import {
  DiagramEdgeData,
  DiagramElementUniqueKey,
  DiagramGroupData,
  DiagramNodeData,
  DiagramNodeDef,
  DiagramStatusIcon,
  ElementHoverMeta,
  Size2D,
} from "@/components/ModelingDiagram/diagram_types";
import {
  ComponentType,
  SchemaVariant,
  UninstalledVariant,
} from "@/api/sdf/dal/schema";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import router from "@/router";
import {
  ComponentDiff,
  ComponentId,
  Edge,
  EdgeId,
  RawComponent,
  RawEdge,
  SocketId,
} from "@/api/sdf/dal/component";
import { Resource } from "@/api/sdf/dal/resource";
import { CodeView } from "@/api/sdf/dal/code_view";
import ComponentUpgrading from "@/components/toasts/ComponentUpgrading.vue";
import { nonNullable } from "@/utils/typescriptLinter";
import handleStoreError from "./errors";
import {
  useChangeSetsStore,
  forceChangeSetApiRequest,
} from "./change_sets.store";
import { useAssetStore } from "./asset.store";
import { useRealtimeStore } from "./realtime/realtime.store";
import { useWorkspacesStore } from "./workspaces.store";
import { useFeatureFlagsStore } from "./feature_flags.store";

export type ComponentNodeId = string;

const toast = useToast();

export type FullComponent = RawComponent & {
  // array of parent IDs
  ancestorIds?: ComponentId[];
  childIds: ComponentId[];
  matchesFilter: boolean;
  icon: IconNames;
  isGroup: false;
  numChildren: number;
  numChildrenResources: number;
};

export type StatusIconsSet = {
  change?: DiagramStatusIcon;
  qualification?: DiagramStatusIcon;
  confirmation?: DiagramStatusIcon;
};

export type ComponentTreeNode = {
  children?: ComponentTreeNode[];
  typeIcon?: string;
  statusIcons?: StatusIconsSet;
} & FullComponent;

export interface CategoryInstalledVariant {
  type: "installed";
  id: string;
  variant: SchemaVariant;
}

export interface CategoryUninstalledVariant {
  type: "uninstalled";
  id: string;
  variant: UninstalledVariant;
}

export type CategoryVariant =
  | CategoryInstalledVariant
  | CategoryUninstalledVariant;

export type Categories = {
  displayName: string;
  schemaVariants: CategoryVariant[];
}[];

export interface AttributeDebugView {
  path: string;
  name: string;
  attributeValueId: string;
  proxyFor?: string | null;
  funcName: string;
  funcId: string;
  funcArgs: { [key: string]: FuncArgDebugView[] } | null;
  visibility: {
    visibility_change_set_pk: string;
    visibility_deleted_at: Date | undefined | null;
  };
  value: object | string | number | boolean | null;
  prototypeId: string;
  prototypeIsComponentSpecific: boolean;
  kind: string;
  view?: string;
}

export interface FuncArgDebugView {
  value: object | string | number | boolean | null;
  name: string;
  valueSource: string;
  valueSourceId: string;
  socketSourceKind: string | null;
  path: string | null;
  isUsed: boolean;
}

export interface SocketDebugView extends AttributeDebugView {
  socketId: string;
  connectionAnnotations: string[];
  inferredConnections: string[];
}

export interface ComponentDebugView {
  name: string;
  schemaVariantId: string;
  attributes: AttributeDebugView[];
  inputSockets: SocketDebugView[];
  outputSockets: SocketDebugView[];
  parentId?: string | null;
}

export interface ComponentGeometry {
  componentId: string;
  position: Vector2d;
  size?: Size2D;
}

export type APIComponentGeometry = {
  x: string;
  y: string;
  width?: string;
  height?: string;
};

type EventBusEvents = {
  deleteSelection: void;
  restoreSelection: void;
  refreshSelectionResource: void;
  eraseSelection: void;
  panToComponent: {
    component: DiagramNodeData | DiagramGroupData;
    center?: boolean;
  };
  rename: ComponentId;
};

export const DEFAULT_COLLAPSED_SIZE = { height: 100, width: 300 };
export const COLLAPSED_HALFWIDTH = DEFAULT_COLLAPSED_SIZE.width / 2;
export const COLLAPSED_HALFHEIGHT = DEFAULT_COLLAPSED_SIZE.height / 2;

export const getAssetIcon = (name: string) => {
  const icons = {
    AWS: "logo-aws",
    "AWS EC2": "logo-aws",
    CoreOS: "logo-coreos",
    Docker: "logo-docker",
    Kubernetes: "logo-k8s",
  } as Record<string, string>;

  let icon = icons[name];

  if (!icon) {
    for (const k in icons) {
      if (name.includes(k)) {
        icon = icons[k];
      }
    }
  }

  return (icon || "logo-si") as IconNames; // fallback to SI logo
};

const edgeFromRawEdge =
  (isInferred: boolean) =>
  (e: RawEdge): Edge => {
    const edge = structuredClone(e) as Edge;
    edge.id = `${edge.toComponentId}_${edge.toSocketId}_${edge.fromSocketId}_${edge.fromComponentId}`;
    edge.isInferred = isInferred;
    return edge;
  };

export const loadCollapsedData = (
  prefix: string,
  key: DiagramElementUniqueKey,
) => {
  const _pos = window.localStorage.getItem(`${prefix}-${key}`);
  if (_pos) {
    return JSON.parse(_pos);
  }
};

export const getCollapsedPrefixes = (workspaceId: string | null) => ({
  SIZE_PREFIX: `${workspaceId}-collapsed-size`,
  POS_PREFIX: `${workspaceId}-collapsed-pos`,
});

const getAncestorIds = (
  allComponents: Record<ComponentId, RawComponent>,
  componentId: ComponentId,
  idsArray = [] as ComponentId[],
): ComponentId[] => {
  const c = allComponents[componentId];

  if (!c) return [];
  const parentId = c.parentId;

  if (parentId) {
    return getAncestorIds(allComponents, parentId, [parentId, ...idsArray]);
  } else {
    return idsArray;
  }
};

export const processRawComponent = (
  component: RawComponent,
  allComponents: Record<ComponentId, RawComponent>,
) => {
  const typeIcon = getAssetIcon(component?.schemaCategory);

  const ancestorIds = getAncestorIds(allComponents, component.id);

  const childIds = [];
  for (const { id: childId, parentId } of _.values(allComponents)) {
    if (component.id === parentId) {
      childIds.push(childId);
    }
  }

  const fullComponent = {
    ...component,
    ancestorIds,
    parentId: _.last(ancestorIds),
    childIds,
    numChildren: 0,
    numChildrenResources: 0,
    icon: typeIcon,
    isGroup: component.componentType !== ComponentType.Component,
  } as FullComponent;

  const nodeDef: DiagramNodeDef = {
    ...fullComponent,
    // swapping "id" to be node id and passing along component id separately for the diagram
    // this is gross and needs to go, but will happen later
    id: fullComponent.id,
    componentId: fullComponent.id,
    title: fullComponent.displayName,
    subtitle: fullComponent.schemaName,
    canBeUpgraded: fullComponent.canBeUpgraded,
    typeIcon: fullComponent?.icon || "logo-si",
  };

  if (nodeDef.componentType === ComponentType.Component) {
    return new DiagramNodeData(nodeDef);
  } else {
    return new DiagramGroupData(nodeDef);
  }
};

const processRawEdge = (
  edge: Edge,
  allComponentsById: Record<ComponentId, DiagramGroupData | DiagramNodeData>,
): DiagramEdgeData | null => {
  const toComponent = allComponentsById[edge.toComponentId];
  if (!allComponentsById[edge.fromComponentId]) return null;
  if (!toComponent) return null;
  else if (!toComponent.def.sockets?.find((s) => s.id === edge.toSocketId))
    return null;
  return new DiagramEdgeData(edge);
};

export const useComponentsStore = (forceChangeSetId?: ChangeSetId) => {
  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspacePk;
  const changeSetsStore = useChangeSetsStore();

  // this needs some work... but we'll probably want a way to force using HEAD
  // so we can load HEAD data in some scenarios while also loading a change set?
  let changeSetId: ChangeSetId | undefined;
  if (forceChangeSetId) {
    changeSetId = forceChangeSetId;
  } else {
    changeSetId = changeSetsStore.selectedChangeSetId;
  }

  // TODO: probably these should be passed in automatically
  // and need to make sure it's done consistently (right now some endpoints vary slightly)
  const visibilityParams = {
    visibility_change_set_pk: changeSetId,
    workspaceId,
  };

  return addStoreHooks(
    workspaceId,
    changeSetId,
    defineStore(
      `ws${workspaceId || "NONE"}/cs${changeSetId || "NONE"}/components`,
      {
        state: () => ({
          // "global" modeling event bus - a bit weird that it lives in the store
          // but we already have global access to it... and this way we can listen to events
          eventBus: mitt<EventBusEvents>(),

          // components within this changeset
          // componentsById: {} as Record<ComponentId, Component>,
          // connectionsById: {} as Record<ConnectionId, Connection>,

          componentCodeViewsById: {} as Record<ComponentId, CodeView[]>,
          componentResourceById: {} as Record<ComponentId, Resource>,
          componentDiffsById: {} as Record<ComponentId, ComponentDiff>,

          rawComponentsById: {} as Record<ComponentId, RawComponent>,
          nodesById: {} as Record<ComponentId, DiagramNodeData>,
          groupsById: {} as Record<ComponentId, DiagramGroupData>,
          allComponentsById: {} as Record<
            ComponentId,
            DiagramNodeData | DiagramGroupData
          >,

          rawEdgesById: {} as Record<EdgeId, Edge>,
          diagramEdgesById: {} as Record<EdgeId, DiagramEdgeData>,
          copyingFrom: null as { x: number; y: number } | null,
          selectedComponentIds: [] as ComponentId[],
          selectedEdgeId: null as EdgeId | null,
          selectedComponentDetailsTab: null as string | null,
          hoveredComponentId: null as ComponentId | null,
          hoveredEdgeId: null as EdgeId | null,
          hoveredComponentMeta: null as ElementHoverMeta | null,

          panTargetComponentId: null as ComponentId | null,

          // used by the diagram to track which schema is selected for
          // insertion. These ids are unique to category variants and
          // can only be used to look up the variant/uninstalled module
          // in `categoryVariantById`
          selectedInsertCategoryVariantId: null as string | null,

          refreshingStatus: {} as Record<ComponentId, boolean>,

          debugDataByComponentId: {} as Record<ComponentId, ComponentDebugView>,
        }),
        getters: {
          // transforming the diagram-y data back into more generic looking data
          // TODO: ideally we just fetch it like this...
          selectedComponentId: (state) => {
            return state.selectedComponentIds.length === 1
              ? state.selectedComponentIds[0]
              : null;
          },
          componentsByParentId(): Record<
            ComponentId,
            (DiagramGroupData | DiagramNodeData)[]
          > {
            return _.groupBy(
              this.allComponentsById,
              (c) => c.def.parentId ?? "root",
            );
          },

          selectedComponent(): DiagramNodeData | DiagramGroupData | undefined {
            return this.allComponentsById[this.selectedComponentId || 0];
          },
          selectedComponents(): (DiagramNodeData | DiagramGroupData)[] {
            return _.compact(
              _.map(
                this.selectedComponentIds,
                (id) => this.allComponentsById[id],
              ),
            );
          },
          selectedEdge(): Edge | undefined {
            return this.rawEdgesById[this.selectedEdgeId || 0];
          },
          hoveredComponent(): DiagramNodeData | DiagramGroupData | undefined {
            return this.allComponentsById[this.hoveredComponentId || 0];
          },

          selectedComponentsAndChildren(): (
            | DiagramNodeData
            | DiagramGroupData
          )[] {
            const selectedAndChildren: Record<
              string,
              DiagramNodeData | DiagramGroupData
            > = {};
            Object.values(this.allComponentsById).forEach((component) => {
              this.selectedComponents?.forEach((el) => {
                if (component.def.ancestorIds?.includes(el.def.id)) {
                  selectedAndChildren[component.def.id] = component;
                }
              });
            });
            this.selectedComponents?.forEach((el) => {
              selectedAndChildren[el.def.id] = el;
            });

            return Object.values(selectedAndChildren);
          },

          deletableSelectedComponents(): (
            | DiagramNodeData
            | DiagramGroupData
          )[] {
            return _.reject(
              this.selectedComponentsAndChildren,
              (c) => c.def.changeStatus === "deleted",
            );
          },
          restorableSelectedComponents(): (
            | DiagramNodeData
            | DiagramGroupData
          )[] {
            return _.filter(
              this.selectedComponents,
              (c) => c.def.changeStatus === "deleted",
            );
          },
          erasableSelectedComponents(): (DiagramNodeData | DiagramGroupData)[] {
            return _.reject(
              this.selectedComponents,
              (c) => c.def.changeStatus === "deleted",
            );
          },
          selectedComponentDiff(): ComponentDiff | undefined {
            return this.componentDiffsById[this.selectedComponentId || 0];
          },
          selectedComponentCode(): CodeView[] | undefined {
            return this.componentCodeViewsById[this.selectedComponentId || 0];
          },
          selectedComponentResource(): Resource | undefined {
            return this.componentResourceById[this.selectedComponentId || 0];
          },

          categories(): Categories {
            const featureFlagsStore = useFeatureFlagsStore();
            const assetStore = useAssetStore();
            const installedGroups = _.groupBy(
              assetStore.variantList,
              "category",
            );
            const uninstalledGroups = featureFlagsStore.ON_DEMAND_ASSETS
              ? _.groupBy(assetStore.uninstalledVariantList, "category")
              : {};

            const mergedKeys = _.uniq([
              ...Object.keys(installedGroups),
              ...Object.keys(uninstalledGroups),
            ]);

            return mergedKeys
              .map((category) => {
                const installedVariants: CategoryInstalledVariant[] =
                  installedGroups[category]
                    ?.filter((v) => v.canCreateNewComponents)
                    .map((v) => ({
                      type: "installed",
                      id: `installed-${v.schemaVariantId}`,
                      variant: v,
                    })) ?? [];

                const uninstalledVariants: CategoryUninstalledVariant[] =
                  uninstalledGroups[category]?.map((v) => ({
                    type: "uninstalled",
                    id: `uninstalled-${v.schemaId}`,
                    variant: v,
                  })) ?? [];

                const schemaVariants: CategoryVariant[] = [
                  ...uninstalledVariants,
                  ...installedVariants,
                ];
                schemaVariants.sort((a, b) =>
                  (
                    a.variant.displayName || a.variant.schemaName
                  )?.localeCompare(
                    b.variant.displayName || b.variant.schemaName,
                  ),
                );

                return {
                  displayName: category,
                  schemaVariants,
                };
              })
              .filter(nonNullable)
              .sort((a, b) => a.displayName.localeCompare(b.displayName));
          },

          // The "category variants", which include both installed and
          // uninstalled, by their unique ids
          categoryVariantById(): { [key: string]: CategoryVariant } {
            return this.categories.reduce((accum, category) => {
              category.schemaVariants.forEach((variant) => {
                accum[variant.id] = variant;
              });
              return accum;
            }, {} as { [key: string]: CategoryVariant });
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
        },
        actions: {
          processRawEdge(edgeId: EdgeId): void {
            const edge = this.rawEdgesById[edgeId];
            if (!edge) return;
            const dEdge = processRawEdge(edge, this.allComponentsById);
            if (dEdge) this.diagramEdgesById[dEdge.def.id] = dEdge;
          },
          processRawComponent(
            componentId: ComponentId,
            processAncestors = true,
          ): void {
            const component = this.rawComponentsById[componentId];
            if (!component) return;
            const elm = processRawComponent(component, this.rawComponentsById);

            // data replacement here
            this.allComponentsById[elm.def.id] = elm;
            // if component changes type it should only be in one group
            // so first remove
            delete this.groupsById[elm.def.id];
            delete this.nodesById[elm.def.id];
            // and then add as appropriate
            if (elm instanceof DiagramGroupData)
              this.groupsById[elm.def.id] = elm;
            else this.nodesById[elm.def.id] = elm;

            // is false when iterating over the whole data set... no need to duplicate work
            if (processAncestors) {
              if (component.parentId) {
                this.processRawComponent(component.parentId, processAncestors);
              }
            }
          },

          async SET_RESOURCE_ID(componentId: ComponentId, resourceId: string) {
            return new ApiRequest<{
              componentId: ComponentId;
            }>({
              method: "post",
              url: "component/set_resource_id",
              params: {
                componentId,
                resourceId,
                ...visibilityParams,
              },
            });
          },

          async RENAME_COMPONENT(componentId: ComponentId, newName: string) {
            return new ApiRequest<{
              componentId: ComponentId;
            }>({
              method: "post",
              url: "component/set_name",
              params: {
                componentId,
                name: newName,
                ...visibilityParams,
              },
            });
          },

          async FETCH_ALL_COMPONENTS() {
            return new ApiRequest<{
              components: RawComponent[];
              edges: RawEdge[];
              inferredEdges: RawEdge[];
            }>({
              method: "get",
              url: "diagram/get_all_components_and_edges",
              params: {
                ...visibilityParams,
              },
              onSuccess: (payload) => {
                this.SET_COMPONENTS_FROM_VIEW(payload);
              },
            });
          },

          SET_COMPONENTS_FROM_VIEW(response: {
            components: RawComponent[];
            edges: RawEdge[];
            inferredEdges: RawEdge[];
          }) {
            // i want to avoid strict assignments here, so i can re-use this
            // this.rawComponentsById = _.keyBy(response.components, "id");
            for (const component of response.components) {
              this.rawComponentsById[component.id] = component;
            }
            // this.allComponentsById = {};
            // this.nodesById = {};
            // this.groupsById = {};
            response.components.forEach((component) => {
              this.processRawComponent(component.id, false);
            });

            const edges =
              response.edges && response.edges.length > 0
                ? response.edges.map(edgeFromRawEdge(false))
                : [];
            const inferred =
              response.inferredEdges && response.inferredEdges.length > 0
                ? response.inferredEdges.map(edgeFromRawEdge(true))
                : [];
            this.rawEdgesById = _.keyBy([...edges, ...inferred], "id");
            Object.keys(this.rawEdgesById).forEach((edgeId) => {
              this.processRawEdge(edgeId);
            });

            // remove invalid component IDs from the selection
            const validComponentIds = _.intersection(
              this.selectedComponentIds,
              _.keys(this.rawComponentsById),
            );
            this.setSelectedComponentId(validComponentIds);
          },

          async FETCH_COMPONENT_DEBUG_VIEW(componentId: ComponentId) {
            return new ApiRequest<ComponentDebugView>({
              url: "component/debug",
              keyRequestStatusBy: componentId,
              params: {
                componentId,
                ...visibilityParams,
              },
              onSuccess: (debugData) => {
                this.debugDataByComponentId[componentId] = debugData;
              },
            });
          },

          setInsertSchema(id: string) {
            this.selectedInsertCategoryVariantId = id;
            this.setSelectedComponentId(null);
          },
          cancelInsert() {
            this.selectedInsertCategoryVariantId = null;
          },

          async CREATE_COMPONENT_CONNECTION(
            from: { componentId: ComponentNodeId; socketId: SocketId },
            to: { componentId: ComponentNodeId; socketId: SocketId },
          ) {
            const timestamp = new Date().toISOString();

            const newEdge = edgeFromRawEdge(false)({
              fromComponentId: from.componentId,
              fromSocketId: from.socketId,
              toComponentId: to.componentId,
              toSocketId: to.socketId,
              toDelete: false,
              changeStatus: "added",
              createdInfo: {
                timestamp,
                actor: { kind: "user", label: "You" },
              },
            });

            return forceChangeSetApiRequest({
              method: "post",
              url: "diagram/create_connection",
              params: {
                fromComponentId: from.componentId,
                fromSocketId: from.socketId,
                toComponentId: to.componentId,
                toSocketId: to.socketId,
                ...visibilityParams,
              },
              onSuccess: () => {},
              optimistic: () => {
                this.rawEdgesById[newEdge.id] = newEdge;
                this.processRawEdge(newEdge.id);

                const replacingEdge = Object.values(this.rawEdgesById)
                  .filter(
                    (e) =>
                      e.isInferred &&
                      e.toSocketId === to.socketId &&
                      e.toComponentId === to.componentId,
                  )
                  .pop();
                if (replacingEdge) {
                  delete this.rawEdgesById[replacingEdge.id];
                  delete this.diagramEdgesById[replacingEdge.id];
                }
                return () => {
                  delete this.rawEdgesById[newEdge.id];
                  if (replacingEdge) {
                    this.rawEdgesById[replacingEdge.id] = replacingEdge;
                    this.processRawEdge(replacingEdge.id);
                  }
                };
              },
            });
          },

          async FETCH_COMPONENT_CODE(componentId: ComponentId) {
            return new ApiRequest<{ codeViews: CodeView[] }>({
              url: "component/get_code",
              keyRequestStatusBy: componentId,
              params: {
                componentId,
                ...visibilityParams,
              },
              onSuccess: (response) => {
                this.componentCodeViewsById[componentId] = response.codeViews;
              },
            });
          },

          async FETCH_COMPONENT_RESOURCE(componentId: ComponentId) {
            return new ApiRequest<{ resource: Resource }>({
              url: "component/get_resource",
              keyRequestStatusBy: componentId,
              params: {
                componentId,
                ...visibilityParams,
              },
              onSuccess: (response) => {
                this.componentResourceById[componentId] = response.resource;
              },
            });
          },

          async FETCH_COMPONENT_JSON(componentId: ComponentId) {
            return new ApiRequest<{ json: unknown }>({
              url: "component/json",
              keyRequestStatusBy: componentId,
              params: {
                componentId,
                ...visibilityParams,
              },
            });
          },

          async FETCH_COMPONENT_DIFF(componentId: ComponentId) {
            return new ApiRequest<{ componentDiff: ComponentDiff }>({
              url: "component/get_diff",
              keyRequestStatusBy: componentId,
              params: {
                componentId,
                ...visibilityParams,
              },
              onSuccess: (response) => {
                this.componentDiffsById[componentId] = response.componentDiff;
              },
            });
          },

          async UPGRADE_COMPONENT(
            componentId: ComponentId,
            componentName: string,
          ) {
            let toastID: number | string;
            return new ApiRequest({
              url: "component/upgrade_component",
              keyRequestStatusBy: componentId,
              method: "post",
              params: {
                componentId,
                ...visibilityParams,
              },
              optimistic: () => {
                toastID = toast({
                  component: ComponentUpgrading,
                  props: {
                    componentName,
                  },
                });
              },
              onSuccess: () => {
                toast.update(toastID, {
                  content: {
                    props: { success: true, componentName },
                    component: ComponentUpgrading,
                  },
                  options: { timeout: 500 },
                });
              },
            });
          },

          async DELETE_EDGE(
            edgeId: EdgeId,
            toSocketId: SocketId,
            fromSocketId: SocketId,
            toComponentId: ComponentId,
            fromComponentId: ComponentId,
          ) {
            return forceChangeSetApiRequest({
              method: "post",
              url: "diagram/delete_connection",
              keyRequestStatusBy: edgeId,
              params: {
                fromSocketId,
                toSocketId,
                toComponentId,
                fromComponentId,
                ...visibilityParams,
              },
              onSuccess: (response) => {
                // this.componentDiffsById[componentId] = response.componentDiff;
              },
              optimistic: () => {
                const edge = this.rawEdgesById[edgeId];

                if (edge?.changeStatus === "added") {
                  const originalEdge = this.rawEdgesById[edgeId];
                  delete this.rawEdgesById[edgeId];
                  delete this.diagramEdgesById[edgeId];
                  this.selectedEdgeId = null;
                  return () => {
                    if (originalEdge) {
                      this.rawEdgesById[edgeId] = originalEdge;
                      this.processRawEdge(edgeId);
                    }
                    this.selectedEdgeId = edgeId;
                  };
                } else if (edge) {
                  const originalStatus = edge.changeStatus;
                  edge.changeStatus = "deleted";
                  edge.deletedInfo = {
                    timestamp: new Date().toISOString(),
                    actor: { kind: "user", label: "You" },
                  };
                  this.rawEdgesById[edgeId] = edge;
                  this.processRawEdge(edgeId);

                  return () => {
                    this.rawEdgesById[edgeId] = {
                      ...edge,
                      changeStatus: originalStatus,
                      deletedInfo: undefined,
                    };
                    this.processRawEdge(edgeId);
                    this.selectedEdgeId = edgeId;
                  };
                }
              },
            });
          },

          async DELETE_COMPONENTS(
            componentIds: ComponentId[],
            forceErase = false,
          ) {
            return forceChangeSetApiRequest<
              Record<ComponentId, boolean>
            >({
              method: "post",
              url: "diagram/delete_components",
              keyRequestStatusBy: componentIds,
              params: {
                componentIds,
                forceErase,
                ...visibilityParams,
              },
              optimistic: () => {
                for (const componentId of componentIds) {
                  const component = this.rawComponentsById[componentId];
                  if (component) {
                    this.rawComponentsById[componentId] = {
                      ...component,
                      changeStatus: "deleted",
                      toDelete: true,
                      deletedInfo: {
                        timestamp: new Date().toISOString(),
                        actor: { kind: "user", label: "You" },
                      },
                    };

                    this.processRawComponent(componentId);
                  }
                }

                // TODO: optimistically delete connected edges?
                // not super important...
                return () => {
                  for (const componentId of componentIds) {
                    const component = this.rawComponentsById[componentId];
                    const originalStatus = component?.changeStatus;
                    if (component && originalStatus) {
                      this.rawComponentsById[componentId] = {
                        ...component,
                        changeStatus: originalStatus,
                        toDelete: false,
                        deletedInfo: undefined,
                      };

                      this.processRawComponent(componentId);
                    }
                  }
                };
              },
            });
          },

          async RESTORE_COMPONENTS(...components: ComponentId[]) {
            return forceChangeSetApiRequest({
              method: "post",
              url: "diagram/remove_delete_intent",
              keyRequestStatusBy: Object.keys(components),
              params: {
                components,
                ...visibilityParams,
              },
              onSuccess: () => {
                for (const componentId of components) {
                  const component = this.rawComponentsById[componentId];
                  if (component) {
                    this.rawComponentsById[componentId] = {
                      ...component,
                      changeStatus: "unmodified",
                      toDelete: false,
                      deletedInfo: undefined,
                    };
                    this.processRawComponent(componentId);
                  }
                }
              },
            });
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

            if (!_.isEqual(router.currentRoute.value.query, newQueryObj)) {
              router.replace({
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
            } else if (ids.length === 1 && ids[0]?.startsWith("e_")) {
              this.selectedComponentIds = [];
              this.selectedEdgeId = ids[0].substring(2);
            } else {
              this.selectedComponentIds = ids.map((id) => id.substring(2));
              this.selectedEdgeId = null;
            }

            const tabSlug =
              (router.currentRoute.value.query?.t as string) || null;
            if (this.selectedComponentIds.length === 1) {
              this.selectedComponentDetailsTab = tabSlug;
            } else {
              this.selectedComponentDetailsTab = null;
            }
          },

          setSelectedEdgeId(selection: EdgeId | null) {
            // clear component selection
            this.selectedComponentIds = [];
            this.selectedEdgeId = selection;
            this.selectedComponentDetailsTab = null;
            this.syncSelectionIntoUrl();
          },
          setSelectedComponentId(
            selection: ComponentId | ComponentId[] | null,
            opts?: { toggle?: boolean; detailsTab?: string },
          ) {
            const key = `${changeSetId}_selected_component`;
            this.selectedEdgeId = null;
            if (!selection || !selection.length) {
              this.selectedComponentIds = [];
              // forget which details tab is active when selection is cleared
              this.selectedComponentDetailsTab = null;
              if (router.currentRoute.value.name === "workspace-compose")
                window.localStorage.removeItem(key);
            } else {
              const validSelectionArray = _.reject(
                _.isArray(selection) ? selection : [selection],
                (id) => !Object.keys(this.allComponentsById).includes(id),
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
          setHoveredComponentId(
            id: ComponentId | null,
            meta?: ElementHoverMeta,
          ) {
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

          async REFRESH_RESOURCE_INFO(componentId: ComponentId) {
            this.refreshingStatus[componentId] = true;
            return new ApiRequest({
              method: "post",
              url: "component/refresh",
              params: {
                componentId,
                workspaceId: visibilityParams.workspaceId,
                visibility_change_set_pk: changeSetsStore.headChangeSetId,
              },
              onSuccess: (response) => {
                // do nothing
              },
            });
          },

          setComponentDisplayName(
            component: DiagramGroupData | DiagramNodeData,
            name: string,
          ) {
            const c = this.rawComponentsById[component.def.id];
            if (!c) return;
            c.displayName = name;
          },
        },
        onActivated() {
          if (!changeSetId) return;

          // TODO: prob want to take loading state into consideration as this will set it before its loaded
          const stopWatchingUrl = watch(
            router.currentRoute,
            () => {
              if (router.currentRoute.value.name === "workspace-compose")
                this.syncUrlIntoSelection();
            },
            {
              immediate: true,
            },
          );

          if (router.currentRoute.value.name === "workspace-compose") {
            const key = `${changeSetId}_selected_component`;
            const lastId = window.localStorage.getItem(key);
            window.localStorage.removeItem(key);
            if (
              lastId &&
              Object.values(this.selectedComponentIds).filter(Boolean)
                .length === 0
            ) {
              this.setSelectedComponentId(lastId);
            }
          }

          // realtime subs
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
                  this.rawComponentsById[data.component.id] = data.component;
                  this.processRawComponent(data.component.id);
                },
              },
              {
                eventType: "ConnectionUpserted",
                callback: async (edge, metadata) => {
                  // If the component that updated wasn't in this change set,
                  // don't update
                  if (metadata.change_set_id !== changeSetId) return;

                  const e = edgeFromRawEdge(false)(edge);
                  this.rawEdgesById[e.id] = e;
                  this.processRawEdge(e.id);
                },
              },
              {
                eventType: "ConnectionDeleted",
                callback: (edge, metadata) => {
                  if (metadata.change_set_id !== changeSetId) return;
                  // making TS happy, we don't need this data since we're just deleting
                  const _edge = edge as RawEdge;
                  _edge.toDelete = true;
                  _edge.createdInfo = {
                    actor: { kind: "system", label: "" },
                    timestamp: "",
                  };
                  const e = edgeFromRawEdge(false)(_edge);
                  delete this.rawEdgesById[e.id];
                  delete this.diagramEdgesById[e.id];
                },
              },
              {
                eventType: "ComponentDeleted",
                callback: (data) => {
                  if (data.changeSetId !== changeSetId) return;
                  delete this.rawComponentsById[data.componentId];
                  delete this.allComponentsById[data.componentId];
                  delete this.nodesById[data.componentId];
                  delete this.groupsById[data.componentId];

                  // remove invalid component IDs from the selection
                  const validComponentIds = _.intersection(
                    this.selectedComponentIds,
                    _.keys(this.rawComponentsById),
                  );
                  this.setSelectedComponentId(validComponentIds);
                },
              },
              {
                eventType: "ComponentUpdated",
                callback: (data, metadata) => {
                  // If the component that updated wasn't in this change set,
                  // don't update
                  if (metadata.change_set_id !== changeSetId) return;
                  this.rawComponentsById[data.component.id] = data.component;
                  this.processRawComponent(data.component.id);

                  if (this.selectedComponentId === data.component.id) {
                    const component = this.rawComponentsById[data.component.id];
                    if (component && component.changeStatus !== "deleted")
                      this.FETCH_COMPONENT_DEBUG_VIEW(data.component.id);
                    else {
                      const idx = this.selectedComponentIds.findIndex(
                        (cId) => cId === data.component.id,
                      );
                      if (idx !== -1) this.selectedComponentIds.slice(idx, 1);
                    }
                  }
                },
              },
              {
                eventType: "InferredEdgeUpsert",
                callback: (data) => {
                  if (data.changeSetId !== changeSetId) return;
                  const edges =
                    data.edges && data.edges.length > 0
                      ? data.edges.map(edgeFromRawEdge(true))
                      : [];
                  for (const edge of edges) {
                    this.rawEdgesById[edge.id] = edge;
                    this.processRawEdge(edge.id);
                  }
                },
              },
              {
                eventType: "InferredEdgeRemove",
                callback: (data) => {
                  if (data.changeSetId !== changeSetId) return;
                  const edges =
                    data.edges && data.edges.length > 0
                      ? data.edges.map(edgeFromRawEdge(true))
                      : [];
                  for (const edge of edges) {
                    delete this.rawEdgesById[edge.id];
                    delete this.diagramEdgesById[edge.id];
                  }
                },
              },
              {
                eventType: "ComponentUpgraded",
                callback: (data) => {
                  // If the component that updated wasn't in this change set,
                  // don't update
                  if (data.changeSetId !== changeSetId) return;
                  // the componentIds ought to be the same, but just in case we'll delete first
                  delete this.rawComponentsById[data.originalComponentId];
                  delete this.allComponentsById[data.originalComponentId];
                  delete this.nodesById[data.originalComponentId];
                  delete this.groupsById[data.originalComponentId];
                  this.rawComponentsById[data.component.id] = data.component;
                  this.processRawComponent(data.component.id);
                  this.setSelectedComponentId(data.component.id);
                },
              },
              {
                eventType: "ResourceRefreshed",
                callback: (data) => {
                  // If the component that updated wasn't in this change set,
                  // don't update
                  if (data.changeSetId !== changeSetId) return;
                  this.rawComponentsById[data.component.id] = data.component;
                  this.processRawComponent(data.component.id);
                  this.refreshingStatus[data.component.id] = false;
                  if (this.selectedComponentId === data.component.id)
                    this.FETCH_COMPONENT_DEBUG_VIEW(data.component.id);
                },
              },
              /* { TODO PUT BACK
              eventType: "DeprecatedActionRunnerReturn",
              callback: (update) => {
                const component = this.componentsById[update.componentId];
                if (!component) return;
                component.hasResource = !!update.resource?.payload;
              },
            }, */
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
                    this.FETCH_ALL_COMPONENTS();
                  }
                },
              },
            ],
          );

          const actionUnsub = this.$onAction(handleStoreError);

          return () => {
            // clear selection without triggering url stuff
            this.selectedComponentIds = [];
            this.selectedEdgeId = null;

            actionUnsub();
            stopWatchingUrl();
            realtimeStore.unsubscribe(`${this.$id}-changeset`);
            realtimeStore.unsubscribe(`${this.$id}-workspace`);
          };
        },
      },
    ),
  )();
};
