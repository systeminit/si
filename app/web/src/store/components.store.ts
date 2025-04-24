import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { Vector2d } from "konva/lib/types";
import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { IconNames } from "@si/vue-lib/design-system";
import { POSITION, useToast } from "vue-toastification";

import mitt from "mitt";
import { connectionAnnotationFitsReference } from "@si/ts-lib";
import { toRaw } from "vue";
import {
  DiagramEdgeData,
  DiagramEdgeDef,
  DiagramElementUniqueKey,
  DiagramGroupData,
  DiagramNodeData,
  DiagramNodeDef,
  DiagramSocketData,
  DiagramSocketDef,
  DiagramSocketDirection,
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
  PotentialConnection,
  RawComponent,
  RawEdge,
  SocketId,
} from "@/api/sdf/dal/component";
import { Resource } from "@/api/sdf/dal/resource";
import { CodeView } from "@/api/sdf/dal/code_view";
import ComponentUpgrading from "@/components/toasts/ComponentUpgrading.vue";
import { nonNullable } from "@/utils/typescriptLinter";
import { ViewId } from "@/api/sdf/dal/views";
import CreatingTemplate from "@/components/toasts/CreatingTemplate.vue";
import handleStoreError from "./errors";
import { useChangeSetsStore } from "./change_sets.store";
import { useAssetStore } from "./asset.store";
import { useRealtimeStore } from "./realtime/realtime.store";
import { useWorkspacesStore } from "./workspaces.store";
import { useFeatureFlagsStore } from "./feature_flags.store";
import { useRouterStore } from "./router.store";
import { useViewsStore } from "./views.store";

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

export type AutoconnectData = {
  componentId: ComponentId;
  componentName: string;
  createdConnections: number;
  potentialConnections: PotentialConnectionData[];
};

export type PotentialConnectionData = {
  socketName: string;
  socketArity: "one" | "many";
  processingConnections: PotentialConnectionMatchData[];
  socketId: SocketId;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  value: any | null;
  attributeValueId: string;
  direction: DiagramSocketDirection;
};

export type PotentialConnectionMatchData = {
  socketName: string;
  socketArity: "one" | "many";
  componentName: string;
  schemaVariantName: string;
  socketId: SocketId;
  componentId: ComponentId;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  value: any | null;
  key: string;
};

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

export type ConnectionMenuStateEntry = {
  componentId: ComponentId;
  socketId: SocketId;
};

export type ConnectionDirection = "output" | "input";

export type ConnectionMenuData = {
  aDirection: ConnectionDirection | undefined;
  A: Partial<ConnectionMenuStateEntry>;
  B: Partial<ConnectionMenuStateEntry>;
};

type EventBusEvents = {
  deleteSelection: void;
  restoreSelection: void;
  refreshSelectionResource: void;
  eraseSelection: void;
  templateFromSelection: void;
  autoconnectComponent: void;
  panToComponent: {
    component: DiagramNodeData | DiagramGroupData;
    center?: boolean;
  };
  setSelection: ComponentId[];
  rename: ComponentId;
  renameView: ViewId;
  openConnectionsMenu: ConnectionMenuData;
};

export const generateSocketPaths = (
  socket: DiagramSocketData,
  viewsStore: ReturnType<typeof useViewsStore>,
): string[] => {
  const socketName = socket.def.label;
  const component = socket.parent;
  const componentName = socket.parent.def.displayName;
  const schemaName = socket.parent.def.schemaName;
  const viewNames = viewsStore.viewNamesByComponentId[component.def.id] ?? [];

  const paths = [] as string[];

  for (const viewName of viewNames) {
    const path = `${viewName}/${schemaName}/${componentName}/${socketName}`;
    paths.push(path);
  }

  return paths;
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
    const edge = structuredClone(toRaw(e)) as Edge;
    edge.id = generateEdgeId(
      edge.fromComponentId,
      edge.toComponentId,
      edge.fromSocketId,
      edge.toSocketId,
    );
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
  schemaName: string;
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
  peerCache: Record<string, PossibleAndExistingPeersLists>,
): PossibleAndExistingPeersLists {
  const cacheKey = `${targetComponentId}-${targetSocket.id}`;
  const cached = peerCache[cacheKey];
  if (cached) {
    return cached;
  }

  const nonDeletedEdges = allEdges.filter(
    (e) => e.def.changeStatus !== "deleted",
  );

  const edgeCountForInputKey = {} as Record<string, number>;
  nonDeletedEdges.forEach((e) => {
    edgeCountForInputKey[e.toSocketKey] ??= 0;
    edgeCountForInputKey[e.toSocketKey] += 1;
  });

  const existingEdges = nonDeletedEdges
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
            // Get only input sockets for output sockets and vice versa
            if (peerSocket.direction === targetSocket.direction) return false;

            // management sockets can only connect to other management sockets
            if (peerSocket.isManagement || targetSocket.isManagement) {
              return !!peerSocket.isManagement && !!targetSocket.isManagement;
            }

            if (peerSocket.direction === "input") {
              const componentAndSocketKey = `${c.uniqueKey}--s-${peerSocket.id}`;
              const edgeCount =
                edgeCountForInputKey[componentAndSocketKey] ?? 0;

              if (
                peerSocket.maxConnections &&
                edgeCount >= peerSocket.maxConnections
              ) {
                return false;
              }
            }

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
            schemaName: c.def.schemaName,
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

  peerCache[cacheKey] = { existingPeers, possiblePeers };

  return { existingPeers, possiblePeers };
}

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

  // insert the schema id into the socket defs, so we can match management
  // sockets
  component.sockets = component.sockets.map((s) => ({
    ...s,
    schemaId: component.schemaId,
  }));

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
  const featureFlagsStore = useFeatureFlagsStore();
  const toComponent = allComponentsById[edge.toComponentId];
  if (!allComponentsById[edge.fromComponentId]) return null;
  if (!toComponent) return null;
  else if (
    !featureFlagsStore.SIMPLE_SOCKET_UI &&
    !toComponent.def.sockets?.find((s) => s.id === edge.toSocketId)
  ) {
    return null;
  }
  return new DiagramEdgeData(edge);
};

export const useComponentsStore = (forceChangeSetId?: ChangeSetId) => {
  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspacePk;
  const changeSetsStore = useChangeSetsStore();
  const featureFlagsStore = useFeatureFlagsStore();
  const routerStore = useRouterStore();
  const realtimeStore = useRealtimeStore();

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

          // used during autoconnect to hold the data to display in a modal.
          autoconnectData: null as AutoconnectData | null,

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

          // TODO: once we remove the flag for new arch (e.g. everyone is using it)
          // We can delete this reference and instead use the values from bifrost
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
            const peerCache = {};
            return (
              targetSocket: DiagramSocketDef,
              targetComponentId: ComponentId,
            ) =>
              getPossibleAndExistingPeerSockets(
                targetSocket,
                targetComponentId,
                allComponents,
                allEdges,
                peerCache,
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

          async AUTOCONNECT_COMPONENT(componentId: ComponentId) {
            return new ApiRequest<{
              created: number;
              potentialIncoming: PotentialConnection[];
            }>({
              url: "component/autoconnect",
              keyRequestStatusBy: componentId,
              method: "post",
              params: {
                componentId,
                ...visibilityParams,
              },
              onSuccess: (payload) => {
                if (payload.potentialIncoming.length === 0) {
                  if (payload.created === 0) {
                    toast("No available connections found for component");
                  } else {
                    toast(`Created ${payload.created} connections`);
                  }
                } else {
                  const thisComponent = this.allComponentsById[componentId];
                  if (!thisComponent) return;
                  this.autoconnectData = {
                    createdConnections: payload.created,
                    potentialConnections: payload.potentialIncoming.map(
                      (pc) => {
                        const socket = thisComponent.def.sockets?.find(
                          (s) => s.id === pc.socketId,
                        );
                        return {
                          socketId: pc.socketId,
                          socketName: socket?.label || "",
                          socketArity:
                            socket?.maxConnections === 1 ? "one" : "many",
                          attributeValueId: pc.attributeValueId,
                          value: pc.value,
                          direction: pc.direction,
                          processingConnections: pc.matches.map(
                            (m: {
                              componentId: string | number;
                              socketId: string;
                              // eslint-disable-next-line @typescript-eslint/no-explicit-any
                              value: any;
                            }) => {
                              const otherComponent =
                                this.allComponentsById[m.componentId];
                              const otherSocket =
                                otherComponent?.def.sockets?.find(
                                  (s) => s.id === m.socketId,
                                );
                              return {
                                socketId: m.socketId,
                                socketName: otherSocket?.label,
                                socketArity:
                                  otherSocket?.maxConnections === 1
                                    ? "one"
                                    : "many",
                                componentId: m.componentId,
                                componentName:
                                  otherComponent?.def.displayName || "",
                                schemaVariantName:
                                  otherComponent?.def.schemaName || "",
                                value: m.value,
                                state: "PENDING",
                                key: `${m.componentId}-${m.socketId}`,
                              } as PotentialConnectionMatchData;
                            },
                          ),
                        } as PotentialConnectionData;
                      },
                    ),
                    componentId,
                    componentName: thisComponent?.def.displayName || "",
                  };
                  this.eventBus.emit("autoconnectComponent");
                }
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
                this.SET_COMPONENTS_FROM_VIEW(payload, {
                  representsAllComponents: true,
                });
              },
            });
          },

          SET_COMPONENTS_FROM_VIEW(
            response: {
              components: RawComponent[];
              edges: RawEdge[];
              inferredEdges: RawEdge[];
              managementEdges: RawEdge[];
            },
            options: { representsAllComponents: boolean } = {
              representsAllComponents: false,
            },
          ) {
            // I need to avoid strict assignments here with the incoming data, only additive and subtractive
            // e.g. operations like this are potentially bad
            // this.rawComponentsById = _.keyBy(response.components, "id");
            // this.allComponentsById = {};

            if (options.representsAllComponents) {
              const existingIds = Object.keys(this.rawComponentsById);
              const allIds = Object.keys(response.components);
              const idsToDelete = existingIds.filter(
                (id) => !allIds.includes(id),
              );
              idsToDelete.forEach((id) => {
                delete this.rawComponentsById[id];
                delete this.allComponentsById[id];
                delete this.nodesById[id];
                delete this.groupsById[id];
              });
            }

            for (const component of response.components) {
              this.rawComponentsById[component.id] = component;
            }

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
              response.managementEdges?.length > 0
                ? response.managementEdges.map(
                    edgeFromRawEdge({ isInferred: false, isManagement: true }),
                  )
                : [];

            const edgesToSet = [...edges, ...inferred, ...management];
            if (options.representsAllComponents) {
              const existingIds = Object.keys(this.rawEdgesById);
              const allIds = Object.keys(edgesToSet);
              const idsToDelete = existingIds.filter(
                (id) => !allIds.includes(id),
              );
              idsToDelete.forEach((id) => {
                delete this.rawEdgesById[id];
              });
            }
            edgesToSet.forEach((edge) => {
              this.rawEdgesById[edge.id] = edge;
              this.processRawEdge(edge.id);
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
            if (changeSetsStore.creatingChangeSet) {
              throw new Error("race, wait until the change set is created");
            }
            if (changeSetId === changeSetsStore.headChangeSetId) {
              changeSetsStore.creatingChangeSet = true;
            }

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
                delete this.diagramEdgesById[newEdge.id];
              },
              optimistic: () => {
                this.rawEdgesById[newEdge.id] = newEdge;
                this.processRawEdge(newEdge.id);
              },
            });
          },

          async OVERRIDE_WITH_CONNECTION(
            from: { componentId: ComponentNodeId; socketId: SocketId },
            to: { componentId: ComponentNodeId; socketId: SocketId },
            attributeValueIdToOverride: string,
          ) {
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
              url: "component/override_with_connection",
              params: {
                fromComponentId: from.componentId,
                fromSocketId: from.socketId,
                toComponentId: to.componentId,
                toSocketId: to.socketId,
                attributeValueIdToOverride,
                ...visibilityParams,
              },
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

          async CREATE_COMPONENT_CONNECTION(
            from: { componentId: ComponentNodeId; socketId: SocketId },
            to: { componentId: ComponentNodeId; socketId: SocketId },
          ) {
            if (changeSetsStore.creatingChangeSet) {
              throw new Error("race, wait until the change set is created");
            }
            if (changeSetId === changeSetsStore.headChangeSetId) {
              changeSetsStore.creatingChangeSet = true;
            }

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
            if (changeSetsStore.creatingChangeSet) {
              throw new Error("race, wait until the change set is created");
            }
            if (changeSetId === changeSetsStore.headChangeSetId) {
              changeSetsStore.creatingChangeSet = true;
            }

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
            if (changeSetsStore.creatingChangeSet) {
              throw new Error("race, wait until the change set is created");
            }
            if (changeSetId === changeSetsStore.headChangeSetId) {
              changeSetsStore.creatingChangeSet = true;
            }

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
            if (changeSetsStore.creatingChangeSet) {
              throw new Error("race, wait until the change set is created");
            }
            if (changeSetId === changeSetsStore.headChangeSetId) {
              changeSetsStore.creatingChangeSet = true;
            }

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
            category: string;
          }) {
            const {
              color,
              assetName,
              funcName,
              componentIds,
              viewId,
              category,
            } = templateData;

            const req = new ApiRequest<{
              schemaVariantId: string;
              funcId: string;
            }>({
              method: "post",
              url: `v2/workspaces/${workspaceId}/change-sets/${changeSetId}/management/generate_template/${viewId}`,
              params: {
                componentIds,
                assetName,
                funcName,
                category,
                color,
              },
              optimistic: (requestUlid) => {
                toast(
                  {
                    component: CreatingTemplate,
                    props: {
                      updating: true,
                    },
                  },
                  {
                    id: requestUlid,
                    timeout: 10 * 1000,
                    closeOnClick: false,
                    position: POSITION.TOP_CENTER,
                    toastClassName: "si-toast-no-defaults",
                  },
                );
              },
              onFail: (_response, requestUlid) => {
                toast.dismiss(requestUlid);
              },
            });

            return req;
          },

          setComponentDisplayName(
            component: DiagramGroupData | DiagramNodeData,
            name: string,
          ) {
            const c = this.rawComponentsById[component.def.id];
            if (!c) return;
            c.displayName = name;
          },

          registerRequestsBegin(requestUlid: string, actionName: string) {
            realtimeStore.inflightRequests.set(requestUlid, actionName);
          },
          registerRequestsEnd(requestUlid: string) {
            realtimeStore.inflightRequests.delete(requestUlid);
          },
        },
        onActivated() {
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

                  data.inferredEdges?.forEach((edge) => {
                    const e = edgeFromRawEdge({
                      isManagement: false,
                      isInferred: true,
                    })(edge);

                    this.rawEdgesById[e.id] = e;
                    this.processRawEdge(e.id);
                  });
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
                  if (oldParent && !data.component.parentId) {
                    this.processAndStoreRawComponent(oldParent, {});
                  }
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
              {
                eventType: "TemplateGenerated",
                callback: (data, metadata) => {
                  if (metadata.change_set_id !== changeSetId) return;

                  // PSA: if the toast doesn't exist one will not be created
                  toast.update(metadata.request_ulid, {
                    content: {
                      props: {
                        updating: false,
                        templateName: data.assetName,
                        schemaVariantId: data.schemaVariantId,
                        funcId: data.funcId,
                        router: (s: string) => {
                          routerStore.push(metadata.change_set_id, {
                            name: "workspace-lab-assets",
                            query: { s },
                          });
                        },
                      },
                      component: CreatingTemplate,
                    },
                    options: { timeout: false, closeOnClick: true },
                  });
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

          const actionUnsub = this.$onAction(handleStoreError);

          return () => {
            actionUnsub();
            realtimeStore.unsubscribe(`${this.$id}-changeset`);
          };
        },
      },
    ),
  )();
};
