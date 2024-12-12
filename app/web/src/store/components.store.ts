import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { Vector2d } from "konva/lib/types";
import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { IconNames } from "@si/vue-lib/design-system";
import { useToast } from "vue-toastification";

import mitt from "mitt";
import { connectionAnnotationFitsReference } from "@si/ts-lib";
import {
  DiagramEdgeData,
  DiagramEdgeDef,
  DiagramElementUniqueKey,
  DiagramGroupData,
  DiagramNodeData,
  DiagramNodeDef,
  DiagramSocketDef,
  DiagramStatusIcon,
  Size2D,
} from "@/components/ModelingDiagram/diagram_types";
import {
  ComponentType,
  SchemaVariant,
  UninstalledVariant,
} from "@/api/sdf/dal/schema";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
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
import { ViewId } from "@/api/sdf/dal/views";
import handleStoreError from "./errors";
import { useChangeSetsStore } from "./change_sets.store";
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
  geometry: {
    [key: string]: {
      id: string;
      created_at: Date;
      updated_at: Date;
      x: number;
      y: number;
      width?: number;
      height?: number;
    };
  };
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
  templateFromSelection: void;
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

export const generateEdgeId = (
  fromComponentId: string,
  toComponentId: string,
  fromSocketId: string,
  toSocketId: string,
) => `${toComponentId}_${toSocketId}_${fromSocketId}_${fromComponentId}`;

const edgeFromRawEdge =
  ({
    isInferred,
    isManagement,
  }: {
    isInferred?: boolean;
    isManagement?: boolean;
  }) =>
  (e: RawEdge): Edge => {
    const edge = structuredClone(e) as Edge;
    if (isManagement) {
      edge.id = `mgmt-${edge.toComponentId}_${edge.fromComponentId}`;
    } else {
      edge.id = generateEdgeId(
        edge.fromComponentId,
        edge.toComponentId,
        edge.fromSocketId,
        edge.toSocketId,
      );
    }
    edge.isInferred = isInferred ?? false;
    edge.isManagement = isManagement ?? false;
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

export type SocketWithParent = DiagramSocketDef & {
  componentName: string;
  componentId: ComponentId;
};

export type SocketWithParentAndEdge = SocketWithParent & {
  edge: DiagramEdgeDef;
};

export interface PossibleAndExistingPeersLists {
  possiblePeers: SocketWithParent[];
  existingPeers: SocketWithParentAndEdge[];
}

// TODO use this in modeling diagram in the drawEdgePossibleTargetSocketKeys computed
export function getPossibleAndExistingPeerSockets(
  targetSocket: DiagramSocketDef,
  targetComponentId: ComponentId,
  allComponents: (DiagramNodeData | DiagramGroupData)[],
  allEdges: DiagramEdgeData[],
): PossibleAndExistingPeersLists {
  const existingEdges = allEdges
    .filter((e) => e.def.changeStatus !== "deleted")
    // map to/from into this/peer to simplify the rest of the algorithm
    .map((edge) =>
      targetSocket.direction === "input"
        ? {
            edge,
            thisComponentId: edge.def.toComponentId,
            thisSocketId: edge.def.toSocketId,
            peerComponentId: edge.def.fromComponentId,
            peerSocketId: edge.def.fromSocketId,
          }
        : {
            edge,
            thisComponentId: edge.def.fromComponentId,
            thisSocketId: edge.def.fromSocketId,
            peerComponentId: edge.def.toComponentId,
            peerSocketId: edge.def.toSocketId,
          },
    )
    // Get only edges relevant to this  socket
    .filter(
      ({ thisComponentId, thisSocketId }) =>
        thisComponentId === targetComponentId &&
        thisSocketId === targetSocket.id,
    );

  const existingPeersIdsAndEdges = existingEdges
    // Create  a set so we can easily search for edges that already exist later
    .reduce((acc, { peerComponentId, peerSocketId, edge }) => {
      acc[`${peerComponentId}-${peerSocketId}`] = edge.def;
      return acc;
    }, {} as Record<string, DiagramEdgeDef>);

  const socketsWithParent = allComponents
    .filter((c) => c.def.id !== targetComponentId)
    .flatMap(
      (c) =>
        c.def.sockets
          ?.filter((peerSocket) => {
            // Management inputs can only be connected to management outputs
            // if the output schema manages the input
            const isManagedSchema =
              peerSocket.schemaId &&
              targetSocket.managedSchemas &&
              targetSocket.managedSchemas.includes(peerSocket.schemaId);

            const isSameSchema = targetSocket.schemaId === peerSocket.schemaId;

            if (peerSocket.isManagement && targetSocket.isManagement) {
              return !!(isSameSchema || isManagedSchema);
            } else if (peerSocket.isManagement || targetSocket.isManagement) {
              return false;
            }

            // Get only input sockets for output sockets and vice versa
            if (peerSocket.direction === targetSocket.direction) return false;

            const [outputCAs, inputCAs] =
              targetSocket.direction === "output"
                ? [
                    targetSocket.connectionAnnotations,
                    peerSocket.connectionAnnotations,
                  ]
                : [
                    peerSocket.connectionAnnotations,
                    targetSocket.connectionAnnotations,
                  ];

            // check socket connection annotations compatibility
            for (const outputCA of outputCAs) {
              for (const inputCA of inputCAs) {
                if (connectionAnnotationFitsReference(outputCA, inputCA)) {
                  return true;
                }
              }
            }

            return false;
          })
          .map((s) => ({
            ...s,
            componentName: c.def.displayName,
            componentId: c.def.id,
          })) ?? [],
    );

  // Partition sockets that are connected and the ones that aren't
  const [existingPeers, possiblePeers] = socketsWithParent.reduce(
    ([existing, possible], socket) => {
      const existingEdge =
        existingPeersIdsAndEdges[`${socket.componentId}-${socket.id}`];

      if (existingEdge) {
        existing.push({ ...socket, edge: existingEdge });
      } else {
        possible.push(socket);
      }

      return [existing, possible];
    },
    [[] as SocketWithParentAndEdge[], [] as SocketWithParent[]],
  );

  return { existingPeers, possiblePeers };
}

export const processRawComponent = (
  component: RawComponent,
  allComponents: Record<ComponentId, RawComponent>,
) => {
  const featureFlagsStore = useFeatureFlagsStore();
  const typeIcon = getAssetIcon(component?.schemaCategory);

  const ancestorIds = getAncestorIds(allComponents, component.id);

  const childIds = [];
  for (const { id: childId, parentId } of _.values(allComponents)) {
    if (component.id === parentId) {
      childIds.push(childId);
    }
  }

  // insert the schema id into the socket defs, so we can match management
  // sockets
  component.sockets = component.sockets.map((s) => ({
    ...s,
    schemaId: component.schemaId,
  }));

  if (!featureFlagsStore.MANAGEMENT_EDGES) {
    component.sockets = component.sockets.filter((s) => !s.isManagement);
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
  const featureFlagsStore = useFeatureFlagsStore();

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
          componentsByParentId(): Record<
            ComponentId,
            (DiagramGroupData | DiagramNodeData)[]
          > {
            return _.groupBy(
              this.allComponentsById,
              (c) => c.def.parentId ?? "root",
            );
          },
          categories(): Categories {
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

          possibleAndExistingPeerSocketsFn: (state) => {
            const allComponents = _.values(state.allComponentsById);
            const allEdges = _.values(state.diagramEdgesById);

            return (
              targetSocket: DiagramSocketDef,
              targetComponentId: ComponentId,
            ) =>
              getPossibleAndExistingPeerSockets(
                targetSocket,
                targetComponentId,
                allComponents,
                allEdges,
              );
          },
        },
        actions: {
          processRawEdge(edgeId: EdgeId): void {
            const edge = this.rawEdgesById[edgeId];
            if (!edge) return;
            const dEdge = processRawEdge(edge, this.allComponentsById);
            if (dEdge) this.diagramEdgesById[dEdge.def.id] = dEdge;
          },
          processAndStoreRawComponent(
            componentId: ComponentId,
            {
              processAncestors = true,
              processChildren = true,
            }: { processAncestors?: boolean; processChildren?: boolean },
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
            if (elm instanceof DiagramGroupData) {
              this.groupsById[elm.def.id] = elm;
            } else {
              this.nodesById[elm.def.id] = elm;
            }

            // is false when iterating over the whole data set... no need to duplicate work
            if (processAncestors) {
              if (component.parentId) {
                this.processAndStoreRawComponent(component.parentId, {
                  processAncestors,
                  processChildren: false,
                });
              }
            }
            if (processChildren) {
              const children = Object.values(this.allComponentsById).filter(
                (c) => c.def.parentId === component.id,
              );
              children.forEach((child) => {
                this.processAndStoreRawComponent(child.def.id, {
                  processAncestors: false,
                  processChildren: true,
                });
              });
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
              managementEdges: RawEdge[];
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
            managementEdges: RawEdge[];
          }) {
            // I want to avoid strict assignments here, so I can re-use
            // this.rawComponentsById = _.keyBy(response.components, "id");
            for (const component of response.components) {
              this.rawComponentsById[component.id] = component;
            }
            // this.allComponentsById = {};
            // this.nodesById = {};
            // this.groupsById = {};
            response.components.forEach((component) => {
              this.processAndStoreRawComponent(component.id, {
                processAncestors: false,
              });
            });

            const edges =
              response.edges && response.edges.length > 0
                ? response.edges.map(
                    edgeFromRawEdge({ isInferred: false, isManagement: false }),
                  )
                : [];
            const inferred =
              response.inferredEdges && response.inferredEdges.length > 0
                ? response.inferredEdges.map(
                    edgeFromRawEdge({ isInferred: true, isManagement: false }),
                  )
                : [];

            const management =
              response.managementEdges?.length > 0 &&
              featureFlagsStore.MANAGEMENT_EDGES
                ? response.managementEdges.map(
                    edgeFromRawEdge({ isInferred: false, isManagement: true }),
                  )
                : [];

            this.rawEdgesById = _.keyBy(
              [...edges, ...inferred, ...management],
              "id",
            );

            Object.keys(this.rawEdgesById).forEach((edgeId) => {
              this.processRawEdge(edgeId);
            });
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
          },
          cancelInsert() {
            this.selectedInsertCategoryVariantId = null;
          },

          async MANAGE_COMPONENT(
            from: { componentId: ComponentNodeId; socketId: SocketId },
            to: { componentId: ComponentNodeId; socketId: SocketId },
          ) {
            if (changeSetsStore.creatingChangeSet)
              throw new Error("race, wait until the change set is created");
            if (changeSetId === changeSetsStore.headChangeSetId)
              changeSetsStore.creatingChangeSet = true;

            const timestamp = new Date().toISOString();

            const newEdge = edgeFromRawEdge({ isManagement: true })({
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

            return new ApiRequest({
              method: "post",
              url: "component/manage",
              params: {
                managerComponentId: from.componentId,
                managedComponentId: to.componentId,
                ...visibilityParams,
              },
              onFail: () => {
                delete this.rawEdgesById[newEdge.id];
              },
              optimistic: () => {
                this.rawEdgesById[newEdge.id] = newEdge;
                this.processRawEdge(newEdge.id);
                return () => {
                  delete this.rawEdgesById[newEdge.id];
                };
              },
            });
          },

          async CREATE_COMPONENT_CONNECTION(
            from: { componentId: ComponentNodeId; socketId: SocketId },
            to: { componentId: ComponentNodeId; socketId: SocketId },
          ) {
            if (changeSetsStore.creatingChangeSet)
              throw new Error("race, wait until the change set is created");
            if (changeSetId === changeSetsStore.headChangeSetId)
              changeSetsStore.creatingChangeSet = true;

            const timestamp = new Date().toISOString();

            const newEdge = edgeFromRawEdge({})({
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

            return new ApiRequest({
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

                const edgesBeingReplaced = Object.values(
                  this.rawEdgesById,
                ).filter(
                  (e) =>
                    e.isInferred &&
                    e.toSocketId === to.socketId &&
                    e.toComponentId === to.componentId,
                );

                for (const edge of edgesBeingReplaced) {
                  delete this.rawEdgesById[edge.id];
                  delete this.diagramEdgesById[edge.id];
                }
                return () => {
                  delete this.rawEdgesById[newEdge.id];
                  for (const edge of edgesBeingReplaced) {
                    delete this.rawEdgesById[edge.id];
                    delete this.diagramEdgesById[edge.id];
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
            if (changeSetsStore.creatingChangeSet)
              throw new Error("race, wait until the change set is created");
            if (changeSetId === changeSetsStore.headChangeSetId)
              changeSetsStore.creatingChangeSet = true;

            const edge = this.rawEdgesById[edgeId];
            const params = edge?.isManagement
              ? {
                  managedComponentId: toComponentId,
                  managerComponentId: fromComponentId,
                  ...visibilityParams,
                }
              : {
                  fromSocketId,
                  toSocketId,
                  toComponentId,
                  fromComponentId,
                  ...visibilityParams,
                };

            const url = edge?.isManagement
              ? "component/unmanage"
              : "diagram/delete_connection";

            return new ApiRequest({
              method: "post",
              url,
              keyRequestStatusBy: edgeId,
              params,
              onSuccess: (response) => {
                // this.componentDiffsById[componentId] = response.componentDiff;
              },
              optimistic: () => {
                if (edge?.changeStatus === "added") {
                  const originalEdge = this.rawEdgesById[edgeId];
                  delete this.rawEdgesById[edgeId];
                  delete this.diagramEdgesById[edgeId];
                  return () => {
                    if (originalEdge) {
                      this.rawEdgesById[edgeId] = originalEdge;
                      this.processRawEdge(edgeId);
                    }
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
                  };
                }
              },
            });
          },

          async DELETE_COMPONENTS(
            componentIds: ComponentId[],
            forceErase = false,
          ) {
            if (changeSetsStore.creatingChangeSet)
              throw new Error("race, wait until the change set is created");
            if (changeSetId === changeSetsStore.headChangeSetId)
              changeSetsStore.creatingChangeSet = true;

            return new ApiRequest<Record<ComponentId, boolean>>({
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

                    this.processAndStoreRawComponent(componentId, {});
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

                      this.processAndStoreRawComponent(componentId, {});
                    }
                  }
                };
              },
            });
          },

          async RESTORE_COMPONENTS(...components: ComponentId[]) {
            if (changeSetsStore.creatingChangeSet)
              throw new Error("race, wait until the change set is created");
            if (changeSetId === changeSetsStore.headChangeSetId)
              changeSetsStore.creatingChangeSet = true;

            return new ApiRequest({
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
                    this.processAndStoreRawComponent(componentId, {});
                  }
                }
              },
            });
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

          async CREATE_TEMPLATE_FUNC_FROM_COMPONENTS(templateData: {
            color: string;
            assetName: string;
            funcName: string;
            componentIds: ComponentId[];
            viewId: ViewId;
          }) {
            const { color, assetName, funcName, componentIds, viewId } =
              templateData;

            return new ApiRequest({
              method: "post",
              url: `v2/workspaces/${workspaceId}/change-sets/${changeSetId}/management/generate_template/${viewId}`,
              params: {
                componentIds,
                assetName,
                funcName,
                category: "Templates",
                color,
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
                  this.processAndStoreRawComponent(data.component.id, {});
                },
              },
              {
                eventType: "ConnectionUpserted",
                callback: async (edge, metadata) => {
                  // If the component that updated wasn't in this change set,
                  // don't update
                  if (metadata.change_set_id !== changeSetId) return;

                  const e = edgeFromRawEdge({
                    isManagement: edge.type === "managementEdge",
                  })(edge);

                  this.rawEdgesById[e.id] = e;
                  this.processRawEdge(e.id);
                },
              },
              {
                eventType: "ConnectionDeleted",
                callback: (edge, metadata) => {
                  if (metadata.change_set_id !== changeSetId) return;

                  let removedEdge: RawEdge;
                  if (edge.type === "attributeValueEdge") {
                    removedEdge = {
                      toDelete: true,
                      createdInfo: {
                        actor: { kind: "system", label: "" },
                        timestamp: "",
                      },
                      fromComponentId: edge.fromComponentId,
                      toComponentId: edge.toComponentId,
                      fromSocketId: edge.fromSocketId,
                      toSocketId: edge.toSocketId,
                    };
                  } else {
                    removedEdge = {
                      toDelete: true,
                      createdInfo: {
                        actor: { kind: "system", label: "" },
                        timestamp: "",
                      },
                      fromComponentId: edge.fromComponentId,
                      toComponentId: edge.toComponentId,
                      fromSocketId: "",
                      toSocketId: "",
                    };
                  }

                  const edgeId = edgeFromRawEdge({
                    isManagement: edge.type === "managementEdge",
                  })(removedEdge).id;

                  delete this.rawEdgesById[edgeId];
                  delete this.diagramEdgesById[edgeId];
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
                },
              },
              {
                eventType: "ComponentUpdated",
                callback: (data, metadata) => {
                  // If the component that updated wasn't in this change set,
                  // don't update
                  if (metadata.change_set_id !== changeSetId) return;
                  const componentId = data.component.id;
                  const oldParent =
                    this.rawComponentsById[componentId]?.parentId;

                  this.rawComponentsById[componentId] = data.component;
                  this.processAndStoreRawComponent(componentId, {});
                  if (oldParent && !data.component.parentId)
                    this.processAndStoreRawComponent(oldParent, {});
                },
              },
              {
                eventType: "InferredEdgeUpsert",
                callback: (data) => {
                  if (data.changeSetId !== changeSetId) return;
                  const edges =
                    data.edges && data.edges.length > 0
                      ? data.edges.map(edgeFromRawEdge({ isInferred: true }))
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
                      ? data.edges.map(edgeFromRawEdge({ isInferred: true }))
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
                  this.processAndStoreRawComponent(data.component.id, {});
                },
              },
              {
                eventType: "ResourceRefreshed",
                callback: (data) => {
                  // If the component that updated wasn't in this change set,
                  // don't update
                  if (data.changeSetId !== changeSetId) return;
                  this.rawComponentsById[data.component.id] = data.component;
                  this.processAndStoreRawComponent(data.component.id, {});
                  this.refreshingStatus[data.component.id] = false;
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
            actionUnsub();
            realtimeStore.unsubscribe(`${this.$id}-changeset`);
            realtimeStore.unsubscribe(`${this.$id}-workspace`);
          };
        },
      },
    ),
  )();
};
