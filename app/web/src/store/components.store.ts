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
  DiagramSocketEdgeData,
  DiagramStatusIcon,
  isDiagramSocketEdgeDef,
  Size2D,
} from "@/components/ModelingDiagram/diagram_types";
import {
  ComponentType,
  SchemaVariant,
  UninstalledVariant,
} from "@/api/sdf/dal/schema";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import {
  AttributePath,
  ComponentTextDiff,
  ComponentId,
  Edge,
  EdgeId,
  isRawSocketEdge,
  isSocketEdge,
  isSubscriptionEdge,
  PotentialConnection,
  RawComponent,
  RawEdge,
  RawSocketEdge,
  RawSubscriptionEdge,
  SocketId,
  SubscriptionEdge,
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
  attributePath: string;
};

export type ConnectionDirection = "output" | "input";

export type ConnectionMenuData = {
  // If true, add a connection without replacing existing ones
  appendConnection?: boolean;
  aDirection: ConnectionDirection | undefined;
  A: Partial<ConnectionMenuStateEntry>;
  B: Partial<ConnectionMenuStateEntry>;
};

export type ComponentsAndEdges = {
  components: RawComponent[];
  edges: RawSocketEdge[];
  inferredEdges: RawSocketEdge[];
  managementEdges: RawSocketEdge[];
  attributeSubscriptionEdges: RawSubscriptionEdge[];
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

// A set of attributes you want to set, with the values you want to set them to.
//
// - SET constant attribute values by putting the path to the attribute you want to set as the key,
//   and the value you want to set it to on the right.
//
//       {
//         "/si/name": "Baby's First Subnet",
//         "/domain/IpAddresses/0": "10.0.0.1",
//         "/domain/Tags/Environment": "production",
//         "/domain/DomainConfig/blah.com/TTL": 3600
//       }
//
// - REPLACE objects/arrays/maps: of special note, if you set an entire array, map or object,
//   it *replaces* its value, and all existing keys are removed or unset. Another way of saying
//   it: after you do this, the attribute on the left will be exactly equal to the value
//   on the right, nothing more, nothing less.
//
//     {
//       "/domain/Tags": { "Environment": "production" },
//       "/domain/IpAddresses": [ "10.0.0.1", "10.0.0.2" ],
//       "/domain/DomainConfig/blah.com": { "TTL": 3600 },
//       "/domain": { "IpAddresses": [ "10.0.0.1" ] }
//     }
//
// - APPEND to array using `-` (or by setting the n+1'th element). If you set an array element
//   that doesn't exist yet, it will be created. `-` is a special syntax for "add a new array
//   element with this value," that doesn't require you to know the (the drawback being you
//   can't append multiple elements to the same array in one API using `-`).
//
//   It is an error to create an array element too far off the end of the array, but you can
//   specify multiple separate elements in order if you want. (It is probably easier to replace
//   the whole array in that case.)
//
//       {
//         "/domain/IpAddresses/0": "10.0.0.0",
//         "/domain/IpAddresses/1": "10.0.0.1",
//         "/domain/IpAddresses/2": "10.0.0.2",
//         "/domain/IpAddresses/-": "10.0.0.3"
//       }
//
// - INSERT to map by setting its value: if you set a map element that hasn't been created yet,
//   it will be created. This will also happen if you set a *field* in a map element that doesn't exist yet (i.e. a
//   map element with object values).
//
//       {
//         "/domain/Tags/Environment": "production",
//         "/domain/DomainConfig/blah.com/TTL": 3600
//       }
//
// - UNSET a value using `{ "$source": null }`. The value will revert to using its default value.
//   (NOTE: `{ "$source": {} }` unsets the value as well, allowing JS callers to construct the
//   API call using `{ "$source": { value: myValueVariable } }``. If myValue is undefined, it
//   will unset the value, but if it is null, it will set the value to null.
//
//       {
//         "/domain/Timeout": { "$source": null },
//         "/domain/DomainConfig/blah.com/TTL": { "$source": "value" }
//       }
//
// - REMOVE an array or map element: unsetting an array or map element will remove it from the
//   array or map. The remaining elements will shift over (it won't "leave a hole").
//
//   *Of note: if you want to remove multiple specific array elements, you should pass them in
//   reverse order.*
//
//       {
//         "/domain/Tags/Environment": { "$source": null },
//         "/domain/IpAddresses/2": { "$source": null },
//         "/domain/IpAddresses/1": { "$source": null }
//       }
//
// - SUBSCRIBE to another attribute's value: this will cause the value to always equal another
//   attribute's value. Components may be specified by their name (which must be globally unique)
//   or ComponentId.
//
//       {
//         "/domain/SubnetId": {
//           "$source": { "component": "ComponentNameOrId", "path": "/resource/SubnetId" }
//         }
//       }
//
//
//      You may specify a function ID to be used in subscription, to transform the value before setting
//      it to the destination AV.
//
//      If no func argument is passed, the func will be si:Identity.
//
//       {
//         "/domain/SubnetId": {
//           "$source": { "component": "ComponentNameOrId", "path": "/resource/SubnetId", "func": "01JWBMRZAANBHKD2G2S5PZQTMA" }
//         }
//       }
//
// - ESCAPE HATCH for setting a value: setting an attribute to `{ "$source": { "value": <value> } }`
//   has the same behavior as all the above cases. The reason this exists is, if you happen to
//   have an object with a "$source" key, the existing interface would treat that as an error.
//   This allows you to set that value anyway.
//
//   This is a safer way to "escape" values if you are writing code that sets values generically
//   without knowing their types and can avoid misinterpreted instructions or possibly even
//   avoid injection attacks.
//
//       {
//         "/domain/Tags": {
//           "$source": {
//             "value": { "$source": "ThisTagIsActuallyNamed_$source" }
//           }
//         }
//       }
//
export type UpdateComponentAttributesArgs = {
  [K in AttributePath]?: AttributeSource;
};

// Set attribute to a subscription (another component's value feeds it)
type AttributeSourceSetSubscription = {
  $source: {
    component: ComponentId | ComponentName;
    path: AttributePath;
    func?: string;
  };
};
const isAttributeSourceSetSubscription = (
  s: AttributeSource,
): s is AttributeSourceSetSubscription => {
  const setPayload = s as AttributeSourceSetSubscription;
  if (!setPayload.$source) return false;

  return "component" in setPayload.$source && "path" in setPayload.$source;
};

// Unset the value with a null value (or empty object/object with undefined for value)
type AttributeSourceUnset =
  | { $source: null }
  | { $source: { value?: undefined } };

const isAttributeSourceUnset = (
  s: AttributeSource,
): s is AttributeSourceUnset => {
  const unsetPayload = s as AttributeSourceUnset;
  return (
    unsetPayload.$source === null || unsetPayload.$source.value === undefined
  );
};

export type AttributeSource =
  | AttributeSourceSetSubscription
  | AttributeSourceUnset
  // Set attribute to a constant JS value (safest way to set to a static value that might contain $source keys)
  | { $source: { value: unknown } }
  // Set attribute to a constant JS value (can be any JSON--object, array, string, number, boolean, null)
  // This is a shorthand for { $source: { value: <value> }}
  | unknown;

// Component name
export type ComponentName = string;

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

export function generateEdgeId(edge: RawEdge): EdgeId {
  if (isRawSocketEdge(edge)) {
    return `${edge.toComponentId}_${edge.toSocketId}_${edge.fromSocketId}_${edge.fromComponentId}`;
  } else {
    return `${edge.toComponentId}_${edge.toAttributePath}_${edge.fromAttributePath}_${edge.fromComponentId}`;
  }
}

function edgeFromRawEdge({
  isInferred,
  isManagement,
}: {
  isInferred?: boolean;
  isManagement?: boolean;
}) {
  return (edge: RawEdge): Edge => {
    return {
      ...structuredClone(toRaw(edge)),
      id: generateEdgeId(edge),
      isInferred: isInferred ?? false,
      isManagement: isManagement ?? false,
    };
  };
}

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
  allEdges: DiagramSocketEdgeData[],
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
    edgeCountForInputKey[e.toSocketKey] =
      (edgeCountForInputKey[e.toSocketKey] ?? 0) + 1;
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

export const useComponentsStore = (forceChangeSetId?: ChangeSetId) => {
  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspacePk;
  const changeSetsStore = useChangeSetsStore();
  const routerStore = useRouterStore();
  const realtimeStore = useRealtimeStore();
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

  const processRawEdge = (
    edge: DiagramEdgeDef,
    allComponentsById: Record<ComponentId, DiagramGroupData | DiagramNodeData>,
  ): DiagramEdgeData | null => {
    const toComponent = allComponentsById[edge.toComponentId];
    if (!allComponentsById[edge.fromComponentId]) return null;
    if (!toComponent) return null;
    if (!featureFlagsStore.SIMPLE_SOCKET_UI) {
      if (!isDiagramSocketEdgeDef(edge)) return null;
      if (!toComponent.def.sockets?.find((s) => s.id === edge.toSocketId)) {
        return null;
      }
    }
    // Create the socket-specific subclass of DiagramSocketEdgeData
    if (isDiagramSocketEdgeDef(edge)) {
      return new DiagramSocketEdgeData(edge);
    } else {
      return new DiagramEdgeData(edge);
    }
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
          componentDiffsById: {} as Record<ComponentId, ComponentTextDiff>,

          rawComponentsById: {} as Record<ComponentId, RawComponent>,
          nodesById: {} as Record<ComponentId, DiagramNodeData>,
          groupsById: {} as Record<ComponentId, DiagramGroupData>,
          allComponentsById: {} as Record<
            ComponentId,
            DiagramNodeData | DiagramGroupData
          >,

          rawEdgesById: {} as Record<EdgeId, Edge>,
          subscriptionEdgesById: {} as Record<EdgeId, SubscriptionEdge>,
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
        }),
        getters: {
          diagramSubscriptionEdgesById(): Record<EdgeId, DiagramEdgeData> {
            const edges = _.values(this.subscriptionEdgesById);

            const rawEdges = _.compact(
              edges.map((edge) => processRawEdge(edge, this.allComponentsById)),
            ).map((edge) => [edge.def.id, edge]);

            return Object.fromEntries(rawEdges);
          },

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
            const uninstalledGroups = _.groupBy(
              assetStore.uninstalledVariantList,
              "category",
            );

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
            const allEdges = _.values(state.diagramEdgesById).filter(
              (e) => e instanceof DiagramSocketEdgeData,
            ) as DiagramSocketEdgeData[]; // TODO upgrade typescript in web app and remove this cast
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
            return new ApiRequest<ComponentsAndEdges>({
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
            response: ComponentsAndEdges,
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

            const subscription =
              featureFlagsStore.SIMPLE_SOCKET_UI &&
              response.attributeSubscriptionEdges?.length > 0
                ? response.attributeSubscriptionEdges.map(
                    edgeFromRawEdge({ isInferred: false, isManagement: false }),
                  )
                : [];

            const edgesToSet = [
              ...edges,
              ...inferred,
              ...management,
              ...subscription,
            ];
            if (options.representsAllComponents) {
              const existingIds = Object.keys(this.rawEdgesById);
              const allIds = Object.keys(edgesToSet);
              const idsToDelete = existingIds.filter(
                (id) => !allIds.includes(id),
              );
              idsToDelete.forEach((id) => {
                delete this.rawEdgesById[id];
              });
              // TODO do this for subscriptions
            }
            edgesToSet.forEach((edge) => {
              this.rawEdgesById[edge.id] = edge;
              this.processRawEdge(edge.id);
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
                    isSocketEdge(e) &&
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
                    isSocketEdge(e) &&
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

          async UPDATE_COMPONENT_ATTRIBUTES(
            componentId: ComponentId,
            payload: UpdateComponentAttributesArgs,
          ) {
            return new ApiRequest({
              method: "put",
              url: `v2/workspaces/${workspaceId}/change-sets/${changeSetId}/components/${componentId}/attributes`,
              params: {
                ...payload,
              },
              passRequestUlidInHeadersOnly: true,
              optimistic: () => {
                for (const toPath in payload) {
                  const update = payload[toPath as AttributePath];
                  if (!update) continue;

                  if (isAttributeSourceSetSubscription(update)) {
                    const newEdge = edgeFromRawEdge({})({
                      fromComponentId: update.$source.component,
                      fromAttributePath: update.$source.path,
                      toComponentId: componentId,
                      toAttributePath: toPath as AttributePath,
                      toDelete: false,
                      changeStatus: "added",
                    });

                    this.rawEdgesById[newEdge.id] = newEdge;
                    this.processRawEdge(newEdge.id);

                    const edgesBeingReplaced = Object.values(
                      this.rawEdgesById,
                    ).filter(
                      (e) =>
                        isSubscriptionEdge(e) &&
                        e.toAttributePath === toPath &&
                        e.toComponentId === componentId,
                    );

                    // TODO Bring back the edges that were deleted by the optimistic call on the callback
                    // Or don't, this won't live much anyway
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
                  }

                  if (isAttributeSourceUnset(update)) {
                    const edgesBeingDeleted = Object.values(
                      this.rawEdgesById,
                    ).filter(
                      (e) =>
                        isSubscriptionEdge(e) &&
                        e.toAttributePath === toPath &&
                        e.toComponentId === componentId,
                    );

                    edgesBeingDeleted.forEach((edge) => {
                      delete this.rawEdgesById[edge.id];
                      delete this.diagramEdgesById[edge.id];
                    });

                    return () => {
                      edgesBeingDeleted.forEach((edge) => {
                        this.rawEdgesById[edge.id] = edge;
                        this.processRawEdge(edge.id);
                      });
                    };
                  }
                }
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
            return new ApiRequest<{ componentDiff: ComponentTextDiff }>({
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

                  let removedEdge: RawSocketEdge;
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
