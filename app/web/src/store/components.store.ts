import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { Vector2d, IRect } from "konva/lib/types";
import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { IconNames } from "@si/vue-lib/design-system";
import { useToast } from "vue-toastification";

import mitt from "mitt";
import { watch } from "vue";
import {
  DiagramEdgeDef,
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
  SchemaVariantId,
} from "@/api/sdf/dal/schema";
import { ChangeSetId, ChangeStatus } from "@/api/sdf/dal/change_set";
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
import { DefaultMap } from "@/utils/defaultmap";
import {
  GROUP_BOTTOM_INTERNAL_PADDING,
  GROUP_DEFAULT_HEIGHT,
  GROUP_DEFAULT_WIDTH,
  GROUP_INTERNAL_PADDING,
} from "@/components/ModelingDiagram/diagram_constants";
import { nonNullable } from "@/utils/typescriptLinter";
import handleStoreError from "./errors";
import { useChangeSetsStore } from "./change_sets.store";
import { useRealtimeStore } from "./realtime/realtime.store";
import { useWorkspacesStore } from "./workspaces.store";

type RequestUlid = string;

export type ComponentNodeId = string;

const toast = useToast();

const MAX_RETRIES = 5;

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

export type Categories = {
  displayName: string;
  schemaVariants: SchemaVariant[];
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

export type SingleSetComponentGeometryData = {
  geometry: ComponentGeometry;
  detach?: boolean;
  newParent?: ComponentId;
};

export type APIComponentGeometry = {
  x: string;
  y: string;
  width?: string;
  height?: string;
};

export type APISingleComponentPosition = {
  geometry: APIComponentGeometry;
  detach: boolean;
  newParent?: ComponentId;
};

type EventBusEvents = {
  deleteSelection: void;
  restoreSelection: void;
  refreshSelectionResource: void;
  eraseSelection: void;
  panToComponent: { componentId: ComponentId; center?: boolean };
};

type PendingComponent = {
  tempId: string;
  position: Vector2d;
  componentId?: ComponentId;
};

export type ComponentData = {
  key: DiagramElementUniqueKey;
  detach?: boolean;
  newParent?: ComponentId;
};

export interface elementPositionAndSize {
  uniqueKey: DiagramElementUniqueKey;
  position?: Vector2d;
  size?: Size2D; // only frames have a size
}

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

  const { SIZE_PREFIX, POS_PREFIX } = getCollapsedPrefixes(workspaceId);

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

          pendingInsertedComponents: {} as Record<string, PendingComponent>,
          collapsedComponents: new Set() as Set<DiagramElementUniqueKey>,
          collapsedElementPositions: {} as Record<
            DiagramElementUniqueKey,
            Vector2d
          >,
          collapsedElementSizes: {} as Record<DiagramElementUniqueKey, Size2D>,

          edgesById: {} as Record<EdgeId, Edge>,
          schemaVariantsById: {} as Record<SchemaVariantId, SchemaVariant>,
          copyingFrom: null as { x: number; y: number } | null,
          selectedComponentIds: [] as ComponentId[],
          selectedEdgeId: null as EdgeId | null,
          selectedComponentDetailsTab: null as string | null,
          hoveredComponentId: null as ComponentId | null,
          hoveredEdgeId: null as EdgeId | null,
          hoveredComponentMeta: null as ElementHoverMeta | null,

          panTargetComponentId: null as ComponentId | null,

          // used by the diagram to track which schema is selected for insertion
          selectedInsertSchemaVariantId: null as SchemaVariantId | null,

          refreshingStatus: {} as Record<ComponentId, boolean>,

          debugDataByComponentId: {} as Record<ComponentId, ComponentDebugView>,

          // Local cache of positions and sizes
          movedElementPositions: {} as Record<
            DiagramElementUniqueKey,
            Vector2d
          >,
          // frames, stored
          resizedElementSizes: {} as Record<DiagramElementUniqueKey, Size2D>,
          // non-frames, measured not stored
          renderedNodeSizes: {} as Record<DiagramElementUniqueKey, Size2D>,

          // size of components when dragged to the stage
          inflightElementSizes: {} as Record<RequestUlid, ComponentId[]>,
          // prevents run away retries, unknown what circumstances could lead to this, but protecting ourselves
          inflightRetryCounter: new DefaultMap<string, number>(() => 0),
        }),
        getters: {
          // transforming the diagram-y data back into more generic looking data
          // TODO: ideally we just fetch it like this...
          selectedComponentId: (state) => {
            return state.selectedComponentIds.length === 1
              ? state.selectedComponentIds[0]
              : null;
          },
          componentsById(): Record<ComponentId, FullComponent> {
            const getAncestorIds = (
              componentId: ComponentId,
              idsArray = [] as ComponentId[],
            ): ComponentId[] => {
              const c = this.rawComponentsById[componentId];

              if (!c) return [];
              const parentId = c.parentId;

              if (parentId) {
                return getAncestorIds(parentId, [parentId, ...idsArray]);
              } else {
                return idsArray;
              }
            };

            const components = _.mapValues(this.rawComponentsById, (rc) => {
              // these categories should probably have a name and a different displayName (ie "aws" vs "Amazon AWS")
              // and eventually can just assume the icon is `logo-${name}`
              const typeIcon = getAssetIcon(rc?.schemaCategory);

              const ancestorIds = getAncestorIds(rc.id);

              const childIds = [];
              for (const { id: childId, parentId } of _.values(
                this.rawComponentsById,
              )) {
                if (rc.id === parentId) {
                  childIds.push(childId);
                }
              }

              return {
                ...rc,
                ancestorIds,
                parentId: _.last(ancestorIds),
                childIds,
                numChildren: 0,
                numChildrenResources: 0,
                icon: typeIcon,
                isGroup: rc.componentType !== ComponentType.Component,
              } as FullComponent;
            });

            const getDeepChildIds = (id: ComponentId): string[] => {
              const component = components[id];
              if (!component?.isGroup) return [];
              return [
                ...(component.childIds ? component.childIds : []),
                ...component.childIds.flatMap(getDeepChildIds),
              ];
            };

            Object.values(components)
              .filter((c) => c.isGroup)
              .forEach((component) => {
                const childIds = getDeepChildIds(component.id);
                component.numChildren = childIds.length;
                component.numChildrenResources = childIds
                  .map((c) => components[c])
                  .filter((c) => c?.hasResource).length;
              });

            return components;
          },
          componentsByParentId(): Record<ComponentId, FullComponent[]> {
            return _.groupBy(this.allComponents, (c) => c.parentId ?? "root");
          },
          parentIdPathByComponentId(): Record<ComponentId, ComponentId[]> {
            const parentsLookup: Record<ComponentId, ComponentId[]> = {};
            // using componentsByParentId to do a tree walk
            const processList = (
              components: FullComponent[],
              parentIds: ComponentId[],
            ) => {
              _.each(components, (c) => {
                parentsLookup[c.id] = parentIds;
                const component = this.componentsByParentId[c.id];
                if (component) {
                  processList(component, [...parentIds, c.id]);
                }
              });
            };
            if (this.componentsByParentId?.root) {
              processList(this.componentsByParentId.root, []);
            }
            return parentsLookup;
          },
          allComponents(): FullComponent[] {
            return _.values(this.componentsById);
          },
          deepChildIdsByComponentId(): Record<ComponentId, ComponentId[]> {
            const getDeepChildIds = (id: ComponentId): string[] => {
              const component = this.componentsById[id];
              if (!component?.isGroup) return [];
              return [
                ...(component.childIds ? component.childIds : []),
                ..._.flatMap(component.childIds, getDeepChildIds),
              ];
            };

            return _.mapValues(this.componentsById, (_component, id) =>
              getDeepChildIds(id),
            );
          },

          allEdges: (state) => _.values(state.edgesById),
          selectedComponent(): FullComponent | undefined {
            return this.componentsById[this.selectedComponentId || 0];
          },
          selectedComponents(): FullComponent[] {
            return _.compact(
              _.map(this.selectedComponentIds, (id) => this.componentsById[id]),
            );
          },
          selectedEdge(): Edge | undefined {
            return this.edgesById[this.selectedEdgeId || 0];
          },
          hoveredComponent(): FullComponent | undefined {
            return this.componentsById[this.hoveredComponentId || 0];
          },

          selectedComponentsAndChildren(): FullComponent[] {
            const selectedAndChildren: Record<string, FullComponent> = {};
            this.allComponents.forEach((component) => {
              this.selectedComponents?.forEach((el) => {
                if (component.ancestorIds?.includes(el.id)) {
                  selectedAndChildren[component.id] = component;
                }
              });
            });
            this.selectedComponents?.forEach((el) => {
              selectedAndChildren[el.id] = el;
            });

            return _.values(selectedAndChildren);
          },

          deletableSelectedComponents(): FullComponent[] {
            return _.reject(
              this.selectedComponentsAndChildren,
              (c) => c.changeStatus === "deleted",
            );
          },
          restorableSelectedComponents(): FullComponent[] {
            return _.filter(
              this.selectedComponents,
              (c) => c.changeStatus === "deleted",
            );
          },
          erasableSelectedComponents(): FullComponent[] {
            return _.reject(
              this.selectedComponents,
              (c) => c.changeStatus === "deleted",
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
          schemaVariantOptionsUnlocked: (
            state,
          ): { label: string; value: string }[] => {
            return Object.values(state.schemaVariantsById)
              .filter((v) => !v.isLocked)
              .map((sv) => ({
                label: sv.displayName || sv.schemaName,
                value: sv.schemaVariantId,
              }));
          },
          schemaVariantOptions: (state): { label: string; value: string }[] => {
            return Object.values(state.schemaVariantsById).map((sv) => ({
              label: sv.displayName || sv.schemaName,
              value: sv.schemaVariantId,
            }));
          },

          diagramNodes(): DiagramNodeDef[] {
            return _.map(this.allComponents, (component) => {
              return {
                ...component,
                // swapping "id" to be node id and passing along component id separately for the diagram
                // this is gross and needs to go, but will happen later
                id: component.id,
                componentId: component.id,
                title: component.displayName,
                subtitle: component.schemaName,
                canBeUpgraded: component.canBeUpgraded,
                typeIcon: component?.icon || "logo-si",
              };
            });
          },
          diagramNodesById(): Record<string, DiagramNodeDef> {
            const r: Record<string, DiagramNodeDef> = {};
            for (const node of this.diagramNodes) {
              r[node.id] = node;
            }
            return r;
          },

          modelIsEmpty(): boolean {
            return !this.diagramNodes.length;
          },
          diagramIsEmpty(): boolean {
            return (
              this.modelIsEmpty && _.isEmpty(this.pendingInsertedComponents)
            );
          },

          diagramEdges(): DiagramEdgeDef[] {
            // filter out edge data if neither component exists
            // or the toComponent Socket doesn't exist
            return this.allEdges.filter((edge) => {
              const toComponent = this.componentsById[edge.toComponentId];
              if (!this.componentsById[edge.fromComponentId]) return false;
              if (!toComponent) return false;
              else if (
                !toComponent.sockets.find((s) => s.id === edge.toSocketId)
              )
                return false;
              return true;
            });
          },

          schemaVariants: (state) => _.values(state.schemaVariantsById),

          categories(): Categories {
            const groups = _.groupBy(this.schemaVariants, "category");
            return Object.keys(groups)
              .map((category) => {
                const variants = groups[category];
                if (!variants) return null;
                return {
                  displayName: category,
                  schemaVariants: variants
                    .filter((v) => v.canCreateNewComponents)
                    .sort((a, b) =>
                      (a.displayName || a.schemaName)?.localeCompare(
                        b.displayName || b.schemaName,
                      ),
                    ),
                };
              })
              .filter(nonNullable)
              .sort((a, b) => a.displayName.localeCompare(b.displayName));
          },

          changeStatsSummary(): Record<ChangeStatus | "total", number> {
            const allChanged = _.filter(
              this.allComponents,
              (c) => !!c.changeStatus,
            );
            const grouped = _.groupBy(allChanged, (c) => c.changeStatus);
            return {
              added: grouped.added?.length || 0,
              deleted: grouped.deleted?.length || 0,
              modified: grouped.modified?.length || 0,
              unmodified: grouped.unmodified?.length || 0,
              total: allChanged.length,
            };
          },

          getDependentComponents: (state) => (componentId: ComponentId) => {
            // TODO: this is ugly... much of this logic is duplicated in ModelingDiagram

            const connectedComponents: Record<ComponentId, ComponentId[]> = {};
            _.each(
              _.values(state.edgesById),
              ({ fromComponentId, toComponentId }) => {
                connectedComponents[fromComponentId] ||= [];
                connectedComponents[fromComponentId]!.push(toComponentId); // eslint-disable-line @typescript-eslint/no-non-null-assertion
              },
            );

            const connectedIds: ComponentId[] = [componentId];

            function walkGraph(id: ComponentId) {
              const nextIds = connectedComponents[id];
              nextIds?.forEach((nid) => {
                if (connectedIds.includes(nid)) return;
                connectedIds.push(nid);
                walkGraph(nid);
              });
            }

            walkGraph(componentId);

            return connectedIds;
          },

          detailsTabSlugs: (state) => {
            const slug = state.selectedComponentDetailsTab;

            // root level tabs
            if (["resource", "actions", "component"].includes(slug || "")) {
              return [slug, undefined];
            }

            // actions tabs are prefixed with "actions-"
            if (slug?.startsWith("actions")) return ["actions", slug];

            // all other subtabs (currently) are in the component tab
            return ["component", slug];
          },

          // The following getters use reported back from the diagram. Don't use to render the diagram.
          // TODO Move these to a diagram stores
          renderedGeometriesByComponentId(): Record<
            ComponentId,
            Vector2d & Size2D
          > {
            const dictionary: Record<ComponentId, Vector2d & Size2D> = {};

            _.forEach(this.componentsById, (c) => {
              let uniqueKey: DiagramElementUniqueKey;
              let size: Size2D;
              if (c.isGroup) {
                uniqueKey = DiagramGroupData.generateUniqueKey(c.id);
                size = this.resizedElementSizes[uniqueKey] ??
                  c.size ?? {
                    width: GROUP_DEFAULT_WIDTH,
                    height: GROUP_DEFAULT_HEIGHT,
                  };
              } else {
                uniqueKey = DiagramNodeData.generateUniqueKey(c.id);

                const renderedSize = this.renderedNodeSizes[uniqueKey];

                if (!renderedSize) return;

                size = renderedSize;
              }

              const position =
                this.movedElementPositions[uniqueKey] ?? c.position;

              dictionary[c.id] = {
                ...position,
                ...size,
              };
            });

            return dictionary;
          },
          // The area that encloses all the components children
          contentBoundingBoxesByGroupId(): Record<ComponentId, IRect> {
            const boxDictionary: Record<string, IRect> = {};
            const groups = this.allComponents.filter((c) => c.isGroup);

            for (const group of groups) {
              const childIds = group.childIds;
              if (!childIds) continue;

              let top;
              let bottom;
              let left;
              let right;

              for (const childId of childIds) {
                const geometry = this.renderedGeometriesByComponentId[childId];
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

              boxDictionary[group.id] = {
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
          combinedElementPositions: (
            state,
          ): Record<DiagramElementUniqueKey, Vector2d> => {
            const pos = _.clone(state.movedElementPositions);
            for (const [key, p] of Object.entries(
              state.collapsedElementPositions,
            )) {
              pos[key] = p;
            }
            return pos;
          },
          combinedElementSizes: (
            state,
          ): Record<DiagramElementUniqueKey, Size2D> => {
            const size = _.clone(state.resizedElementSizes);
            for (const [key, s] of Object.entries(
              state.collapsedElementSizes,
            )) {
              size[key] = s;
            }
            return size;
          },
        },
        actions: {
          expandComponents(...keys: DiagramElementUniqueKey[]) {
            keys.forEach((key) => {
              this.collapsedComponents.delete(key);
              delete this.collapsedElementPositions[key];
              delete this.collapsedElementSizes[key];
            });
            this.persistCollapsed();
          },

          persistCollapsed() {
            window.localStorage.setItem(
              `${workspaceId}-collapsed-components`,
              JSON.stringify(Array.from(this.collapsedComponents)),
            );
          },

          removeCollapsedData(key: DiagramElementUniqueKey) {
            // TODO: rework if this ends up being expensive...
            const { SIZE_PREFIX, POS_PREFIX } =
              getCollapsedPrefixes(workspaceId);
            window.localStorage.removeItem(`${SIZE_PREFIX}-${key}`);
            window.localStorage.removeItem(`${POS_PREFIX}-${key}`);
          },

          initMinimzedElementPositionAndSize(key: DiagramElementUniqueKey) {
            const { SIZE_PREFIX, POS_PREFIX } =
              getCollapsedPrefixes(workspaceId);
            let position;
            let size;
            position = loadCollapsedData(POS_PREFIX, key) as
              | Vector2d
              | undefined;
            if (!position) position = this.combinedElementPositions[key];
            size = loadCollapsedData(SIZE_PREFIX, key) as Size2D | undefined;
            if (!size)
              size = this.collapsedElementSizes[key] || DEFAULT_COLLAPSED_SIZE;
            return { position, size };
          },

          updateMinimzedElementPositionAndSize(
            ...elms: elementPositionAndSize[]
          ) {
            elms.forEach((elm) => {
              this.collapsedComponents.add(elm.uniqueKey);
              if (elm.size) {
                this.collapsedElementSizes[elm.uniqueKey] = elm.size;
                window.localStorage.setItem(
                  `${SIZE_PREFIX}-${elm.uniqueKey}`,
                  JSON.stringify(elm.size),
                );
              }
              if (elm.position) {
                this.collapsedElementPositions[elm.uniqueKey] = elm.position;
                window.localStorage.setItem(
                  `${POS_PREFIX}-${elm.uniqueKey}`,
                  JSON.stringify(elm.position),
                );
              }
            });
            this.persistCollapsed();
          },
          // actually fetches diagram-style data, but we have a computed getter to turn back into more generic component data above
          async FETCH_DIAGRAM_DATA() {
            return new ApiRequest<{
              components: RawComponent[];
              edges: RawEdge[];
              inferredEdges: RawEdge[];
            }>({
              url: "diagram/get_diagram",
              params: {
                ...visibilityParams,
              },
              onSuccess: (response) => {
                this.resizedElementSizes = {};
                this.movedElementPositions = {};

                this.rawComponentsById = _.keyBy(response.components, "id");
                const edges =
                  response.edges && response.edges.length > 0
                    ? response.edges.map(edgeFromRawEdge(false))
                    : [];
                const inferred =
                  response.inferredEdges && response.inferredEdges.length > 0
                    ? response.inferredEdges.map(edgeFromRawEdge(true))
                    : [];
                this.edgesById = _.keyBy([...edges, ...inferred], "id");

                // remove invalid component IDs from the selection
                const validComponentIds = _.intersection(
                  this.selectedComponentIds,
                  _.keys(this.rawComponentsById),
                );
                this.setSelectedComponentId(validComponentIds);

                // find any pending inserts that we know the component id of
                // and have now been loaded - and remove them from the pending inserts
                const pendingInsertsByComponentId = _.keyBy(
                  this.pendingInsertedComponents,
                  (p) => p.componentId || "",
                );
                const pendingComponentIdsThatAreComplete = _.compact(
                  _.intersection(
                    _.map(this.pendingInsertedComponents, (p) => p.componentId),
                    _.keys(this.rawComponentsById),
                  ),
                );
                _.each(pendingComponentIdsThatAreComplete, (id) => {
                  const tempId = pendingInsertsByComponentId[id]?.tempId;
                  if (tempId) delete this.pendingInsertedComponents[tempId];
                }); // and set the selection to the new component
                if (pendingComponentIdsThatAreComplete[0]) {
                  this.setSelectedComponentId(
                    pendingComponentIdsThatAreComplete[0],
                  );
                }
              },
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

          async FETCH_AVAILABLE_SCHEMAS() {
            return new ApiRequest<Array<SchemaVariant>>({
              url: [
                "v2",
                "workspaces",
                { workspaceId },
                "change-sets",
                { changeSetId },
                "schema-variants",
              ],
              params: {
                ...visibilityParams,
              },
              onSuccess: (response) => {
                this.schemaVariantsById = _.keyBy(response, "schemaVariantId");
              },
            });
          },

          constructGeometryData(componentData: ComponentData[]) {
            const componentUpdate: SingleSetComponentGeometryData[] = [];
            for (const { key, detach, newParent } of componentData) {
              const position = this.movedElementPositions[key];
              if (position) {
                position.x = Math.round(position.x);
                position.y = Math.round(position.y);
              }
              const size = this.resizedElementSizes[key];
              if (size) {
                size.width = Math.round(size.width);
                size.height = Math.round(size.height);
              }
              const componentId = DiagramNodeData.componentIdFromUniqueKey(
                DiagramGroupData.componentIdFromUniqueKey(key),
              );
              if (position && componentId) {
                componentUpdate.push({
                  geometry: {
                    componentId,
                    position,
                    size,
                  },
                  detach,
                  newParent,
                });
              }
            }
            return componentUpdate;
          },

          async SET_COMPONENT_GEOMETRY(
            componentUpdates: SingleSetComponentGeometryData[],
          ) {
            if (changeSetsStore.creatingChangeSet)
              throw new Error("race, wait until the change set is created");
            if (changeSetId === changeSetsStore.headChangeSetId)
              changeSetsStore.creatingChangeSet = true;

            const dataByComponentId: Record<
              ComponentId,
              APISingleComponentPosition
            > = {};

            componentUpdates.forEach((p) => {
              dataByComponentId[p.geometry.componentId] = {
                detach: !!p.detach,
                newParent: p.newParent,
                geometry: {
                  x: p.geometry.position.x.toString(),
                  y: p.geometry.position.y.toString(),
                  width: p.geometry.size?.width.toString(),
                  height: p.geometry.size?.height.toString(),
                },
              };
            });

            return new ApiRequest<{ requestUlid: RequestUlid }>({
              method: "post",
              url: "diagram/set_component_position",
              params: {
                dataByComponentId,
                diagramKind: "configuration",
                ...visibilityParams,
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

                const maybe_retry = failed.filter(
                  (x) => !all_inflight_components.has(x),
                );

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
                  const components = [] as ComponentData[];
                  for (const componentId of retry) {
                    const c = this.rawComponentsById[componentId];
                    if (!c) continue;

                    const node = this.diagramNodesById[c.id];
                    if (!node) continue;

                    let typedNode: DiagramNodeData | DiagramGroupData;
                    if (c.componentType === ComponentType.Component) {
                      typedNode = new DiagramNodeData(node);
                    } else {
                      typedNode = new DiagramGroupData(node);
                    }

                    const newParent = typedNode.def.parentId;
                    const detach = !newParent;
                    components.push({
                      key: typedNode.uniqueKey,
                      newParent,
                      detach,
                    });
                  }
                  const payload = this.constructGeometryData(components);
                  this.SET_COMPONENT_GEOMETRY(payload);
                }
              },
              onSuccess: (response) => {
                delete this.inflightElementSizes[response.requestUlid];
              },
              optimistic: (requestUlid: RequestUlid) => {
                this.inflightElementSizes[requestUlid] =
                  Object.keys(dataByComponentId);

                const prevParents: Record<
                  ComponentId,
                  ComponentId | undefined
                > = {};
                for (const {
                  geometry,
                  detach,
                  newParent,
                } of componentUpdates) {
                  const componentId = geometry.componentId;
                  const component = this.rawComponentsById[componentId];

                  if (detach) {
                    if (!component) return;
                    prevParents[component.id] = component?.parentId;
                    component.parentId = undefined;

                    // remove inferred edges between children and parents
                    const full_component = this.componentsById[componentId];
                    for (const edge of _.filter(
                      _.values(this.edgesById),
                      (edge) =>
                        !!(
                          edge.isInferred &&
                          edge.toComponentId === component.id &&
                          full_component?.ancestorIds?.includes(
                            edge.fromComponentId,
                          )
                        ),
                    )) {
                      delete this.edgesById[edge.id];
                    }
                  }

                  if (newParent) {
                    const component = this.rawComponentsById[componentId];
                    if (!component) continue;

                    prevParents[component.id] = component.parentId;

                    component.parentId = newParent;
                  }
                }

                // NOTE: `onDragElementsMove` only looks at parentId
                // so we don't have to manipulate `ancestorIds` here
                return () => {
                  for (const componentId in prevParents) {
                    const component = this.rawComponentsById[componentId];
                    if (component)
                      component.parentId = prevParents[componentId];
                  }
                };
              },
            });
          },

          setInsertSchema(schemaVariantId: SchemaVariantId) {
            this.selectedInsertSchemaVariantId = schemaVariantId;
            this.setSelectedComponentId(null);
          },
          cancelInsert() {
            this.selectedInsertSchemaVariantId = null;
          },

          async CREATE_COMPONENT(
            schemaVariantId: SchemaVariantId,
            position: Vector2d,
            parentId?: string,
            size?: Size2D,
          ) {
            if (changeSetsStore.creatingChangeSet)
              throw new Error("race, wait until the change set is created");
            if (changeSetId === changeSetsStore.headChangeSetId)
              changeSetsStore.creatingChangeSet = true;

            const tempInsertId = _.uniqueId("temp-insert-component");

            return new ApiRequest<{
              componentId: ComponentId;
              nodeId: ComponentNodeId;
            }>({
              method: "post",
              url: "diagram/create_component",
              headers: { accept: "application/json" },
              params: {
                schemaVariantId,
                parentId,
                x: position.x.toString(),
                y: position.y.toString(),
                height: size?.height.toString(),
                width: size?.width.toString(),
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
          async CREATE_COMPONENT_CONNECTION(
            from: { componentId: ComponentNodeId; socketId: SocketId },
            to: { componentId: ComponentNodeId; socketId: SocketId },
          ) {
            if (changeSetsStore.creatingChangeSet)
              throw new Error("race, wait until the change set is created");
            if (changeSetId === changeSetsStore.headChangeSetId)
              changeSetsStore.creatingChangeSet = true;

            const tempId = `temp-edge-${+new Date()}`;

            return new ApiRequest<{
              id: string;
              creatd_by: string | null;
              deleted_by: string | null;
            }>({
              method: "post",
              url: "diagram/create_connection",
              params: {
                fromComponentId: from.componentId,
                fromSocketId: from.socketId,
                toComponentId: to.componentId,
                toSocketId: to.socketId,
                ...visibilityParams,
              },
              onSuccess: (response) => {
                // change our temporary id to the real one, only if we haven't re-fetched the diagram yet
                if (this.edgesById[tempId]) {
                  const edge = this.edgesById[tempId];
                  if (edge) {
                    edge.id = response.id;
                    this.edgesById[response.id] = edge;
                    delete this.edgesById[tempId];
                  }
                }
                // TODO: store component details rather than waiting for re-fetch
              },
              optimistic: () => {
                const nowTs = new Date().toISOString();
                this.edgesById[tempId] = {
                  id: tempId,
                  fromComponentId: from.componentId,
                  fromSocketId: from.socketId,
                  toComponentId: to.componentId,
                  toSocketId: to.socketId,
                  changeStatus: "added",
                  toDelete: false,
                  isInferred: false,
                  createdInfo: {
                    timestamp: nowTs,
                    actor: { kind: "user", label: "You" },
                  },
                };

                const replacingEdge = this.allEdges
                  .filter(
                    (e) =>
                      e.isInferred &&
                      e.toSocketId === to.socketId &&
                      e.toComponentId === to.componentId,
                  )
                  .pop();
                if (replacingEdge) {
                  delete this.edgesById[replacingEdge.id];
                }
                return () => {
                  delete this.edgesById[tempId];
                  if (replacingEdge) {
                    this.edgesById[replacingEdge.id] = replacingEdge;
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

            return new ApiRequest({
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
                const edge = this.edgesById[edgeId];

                if (edge?.changeStatus === "added") {
                  const originalEdge = this.edgesById[edgeId];
                  delete this.edgesById[edgeId];
                  this.selectedEdgeId = null;
                  return () => {
                    if (originalEdge) {
                      this.edgesById[edgeId] = originalEdge;
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
                  this.edgesById[edgeId] = edge;

                  return () => {
                    this.edgesById[edgeId] = {
                      ...edge,
                      changeStatus: originalStatus,
                      deletedInfo: undefined,
                    };
                    this.selectedEdgeId = edgeId;
                  };
                }
              },
            });
          },

          async PASTE_COMPONENTS(
            components: {
              id: ComponentId;
              componentGeometry: Vector2d & Size2D;
            }[],
            newParentNodeId?: ComponentNodeId,
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
              url: "diagram/paste_components",
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
                    }
                  }
                };
              },
            });
          },

          async RESTORE_COMPONENTS(components: Record<ComponentId, boolean>) {
            if (changeSetsStore.creatingChangeSet)
              throw new Error("race, wait until the change set is created");
            if (changeSetId === changeSetsStore.headChangeSetId)
              changeSetsStore.creatingChangeSet = true;

            const payload = [];
            for (const [key, value] of Object.entries(components)) {
              payload.push({ componentId: key, fromBaseChangeSet: value });
            }
            return new ApiRequest({
              method: "post",
              url: "diagram/remove_delete_intent",
              keyRequestStatusBy: Object.keys(components),
              params: {
                components: payload,
                ...visibilityParams,
              },
              onSuccess: () => {
                for (const componentId of Object.keys(components)) {
                  const component = this.rawComponentsById[componentId];
                  if (component) {
                    this.rawComponentsById[componentId] = {
                      ...component,
                      changeStatus: "unmodified",
                      toDelete: false,
                      fromBaseChangeSet: false,
                      deletedInfo: undefined,
                    };
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
                (id) => !this.componentsById[id],
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

          setComponentDisplayName(component: FullComponent, name: string) {
            const c = this.rawComponentsById[component.id];
            if (!c) return;
            c.displayName = name;
          },
        },
        onActivated() {
          if (!changeSetId) return;

          try {
            const minimzedString = window.localStorage.getItem(
              `${workspaceId}-collapsed-components`,
            );
            if (minimzedString) {
              const collapsed = JSON.parse(minimzedString);
              this.collapsedComponents = new Set(collapsed);
            }
          } catch (e) {
            /* empty */
          }

          this.collapsedComponents.forEach((key) => {
            this.collapsedElementPositions[key] = loadCollapsedData(
              POS_PREFIX,
              key,
            );
            this.collapsedElementSizes[key] = loadCollapsedData(
              SIZE_PREFIX,
              key,
            );
          });

          // trigger initial load
          this.FETCH_DIAGRAM_DATA();
          this.FETCH_AVAILABLE_SCHEMAS();

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
                },
              },
              {
                eventType: "ConnectionUpserted",
                callback: async (edge, metadata) => {
                  // If the component that updated wasn't in this change set,
                  // don't update
                  if (metadata.change_set_id !== changeSetId) return;
                  const e = edgeFromRawEdge(false)(edge);
                  this.edgesById[e.id] = e;
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
                  delete this.edgesById[e.id];
                },
              },
              {
                eventType: "ComponentDeleted",
                callback: (data) => {
                  if (data.changeSetId !== changeSetId) return;
                  delete this.rawComponentsById[data.componentId];

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
                    this.edgesById[edge.id] = edge;
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
                    delete this.edgesById[edge.id];
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
                  this.rawComponentsById[data.component.id] = data.component;
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
                  this.refreshingStatus[data.component.id] = false;
                  if (this.selectedComponentId === data.component.id)
                    this.FETCH_COMPONENT_DEBUG_VIEW(data.component.id);
                },
              },
              {
                eventType: "SchemaVariantUpdated",
                callback: (variant, metadata) => {
                  if (metadata.change_set_id !== changeSetId) return;
                  this.schemaVariantsById[variant.schemaVariantId] = variant;
                },
              },
              {
                eventType: "SchemaVariantCreated",
                callback: (variant, metadata) => {
                  if (metadata.change_set_id !== changeSetId) return;
                  this.schemaVariantsById[variant.schemaVariantId] = variant;
                },
              },
              {
                eventType: "SchemaVariantDeleted",
                callback: (data, metadata) => {
                  if (metadata.change_set_id !== changeSetId) return;
                  delete this.schemaVariantsById[data.schemaVariantId];
                },
              },
              {
                eventType: "ModuleImported",
                callback: (schemaVariants, metadata) => {
                  if (metadata.change_set_id !== changeSetId) return;
                  for (const variant of schemaVariants) {
                    this.schemaVariantsById[variant.schemaVariantId] = variant;
                  }
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
                    this.FETCH_DIAGRAM_DATA();
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
