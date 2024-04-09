import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { Vector2d } from "konva/lib/types";
import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { IconNames } from "@si/vue-lib/design-system";

import mitt from "mitt";
import { watch } from "vue";
import {
  ComponentType,
  DiagramEdgeDef,
  DiagramNodeDef,
  DiagramSocketDef,
  DiagramStatusIcon,
  ElementHoverMeta,
  GridPoint,
  Size2D,
} from "@/components/ModelingDiagram/diagram_types";
import {
  DiagramNode,
  DiagramSchema,
  DiagramSchemaVariant,
} from "@/api/sdf/dal/diagram";
import {
  ChangeStatus,
  ComponentStats,
  ChangeSetId,
} from "@/api/sdf/dal/change_set";
import router from "@/router";
import { ComponentDiff } from "@/api/sdf/dal/component";
import { Resource } from "@/api/sdf/dal/resource";
import { CodeView } from "@/api/sdf/dal/code_view";
import { ActorView } from "@/api/sdf/dal/history_actor";
import { useChangeSetsStore } from "./change_sets.store";
import { useRealtimeStore } from "./realtime/realtime.store";
import {
  QualificationStatus,
  useQualificationsStore,
} from "./qualifications.store";
import { useWorkspacesStore } from "./workspaces.store";
import { useStatusStore } from "./status.store";

export type ComponentId = string;
export type ComponentNodeId = string;
export type EdgeId = string;
export type SocketId = string;
type SchemaId = string;

type RawComponent = {
  changeStatus: ChangeStatus;
  color: string;
  createdInfo: ActorAndTimestamp;
  deletedInfo?: ActorAndTimestamp;
  displayName: string;
  id: ComponentId;
  componentType: ComponentType;
  parentId?: ComponentId;
  position: GridPoint;
  size?: Size2D;
  hasResource: boolean;
  schemaCategory: string;
  schemaId: string; // TODO: probably want to move this to a different store and not load it all the time
  schemaName: string;
  schemaVariantId: string;
  schemaVariantName: string;
  sockets: DiagramSocketDef[];
  updatedInfo: ActorAndTimestamp;
  toDelete: boolean;
};

export type FullComponent = RawComponent & {
  // array of parent IDs
  ancestorIds?: ComponentId[];
  childIds: ComponentId[];
  matchesFilter: boolean;
  icon: IconNames;
  isGroup: false;
};

type Edge = {
  id: EdgeId;
  fromComponentId: ComponentId;
  fromSocketId: SocketId;
  toComponentId: ComponentId;
  toSocketId: SocketId;
  isInvisible?: boolean;
  /** change status of edge in relation to head */
  changeStatus?: ChangeStatus;
  createdInfo: ActorAndTimestamp;
  // updatedInfo?: ActorAndTimestamp; // currently we dont ever update an edge...
  deletedInfo?: ActorAndTimestamp;
  toDelete: boolean;
};

export interface ActorAndTimestamp {
  actor: ActorView;
  timestamp: string;
}

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

export interface DiagramSchemaVariantWithDisplayMetadata
  extends DiagramSchemaVariant {
  schemaName: string;
}

export interface DiagramSchemaWithDisplayMetadata extends DiagramSchema {
  category: string;
  color: string;
}

export type Categories = {
  displayName: string;
  schemas: DiagramSchemaWithDisplayMetadata[];
}[];

const qualificationStatusToIconMap: Record<
  QualificationStatus | "notexists",
  DiagramStatusIcon
> = {
  success: { icon: "check-hex-outline", tone: "success" },
  warning: { icon: "check-hex-outline", tone: "warning" },
  failure: { icon: "x-hex-outline", tone: "error" },
  running: { icon: "loader", tone: "info" },
  notexists: { icon: "none" },
};

export interface AttributeDebugView {
  path: string;
  name: string;
  attributeValueId: string;
  proxyFor?: string | null;
  funcName: string;
  funcId: string;
  funcArgs: object;
  argSources: { [key: string]: string | null } | null;
  visibility: {
    visibility_change_set_pk: string;
    visibility_deleted_at: Date | undefined | null;
  };
  value: object | string | number | boolean | null;
  prototypeId: string;
  kind: string;
  materializedView?: string;
}
export interface SocketDebugView extends AttributeDebugView {
  socketId: string;
  connectionAnnotations: string[];
}

export interface ComponentDebugView {
  name: string;
  schemaVariantId: string;
  attributes: AttributeDebugView[];
  inputSockets: SocketDebugView[];
  outputSockets: SocketDebugView[];
}

type EventBusEvents = {
  deleteSelection: void;
  restoreSelection: void;
  refreshSelectionResource: void;
  panToComponent: { componentId: ComponentId; center?: boolean };
};

type PendingComponent = {
  tempId: string;
  position: Vector2d;
  componentId?: ComponentId;
};

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

          edgesById: {} as Record<EdgeId, Edge>,
          schemasById: {} as Record<SchemaId, DiagramSchema>,
          copyingFrom: null as { x: number; y: number } | null,
          selectedComponentIds: [] as ComponentId[],
          selectedEdgeId: null as EdgeId | null,
          selectedComponentDetailsTab: null as string | null,
          hoveredComponentId: null as ComponentId | null,
          hoveredEdgeId: null as EdgeId | null,
          hoveredComponentMeta: null as ElementHoverMeta | null,

          panTargetComponentId: null as ComponentId | null,

          // used by the diagram to track which schema is selected for insertion
          selectedInsertSchemaId: null as SchemaId | null,

          refreshingStatus: {} as Record<ComponentId, boolean>,

          debugDataByComponentId: {} as Record<ComponentId, ComponentDebugView>,

          pastingId: null as string | null,
          pastingError: undefined as string | undefined,
          pastingLoading: false as boolean,
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

              if (!c) throw new Error("what?");
              const parentId = c.parentId;

              if (parentId) {
                return getAncestorIds(parentId, [parentId, ...idsArray]);
              } else {
                return idsArray;
              }
            };

            return _.mapValues(this.rawComponentsById, (rc) => {
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
                icon: typeIcon,
                isGroup: rc.componentType !== ComponentType.Component,
              } as FullComponent;
            });
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

          deletableSelectedComponents(): FullComponent[] {
            return _.reject(
              this.selectedComponents,
              (c) => c.changeStatus === "deleted",
            );
          },
          restorableSelectedComponents(): FullComponent[] {
            return _.filter(
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

          diagramNodes(): DiagramNodeDef[] {
            const qualificationsStore = useQualificationsStore();
            const statusStore = useStatusStore();

            // adding logo and qualification info into the nodes
            // TODO: probably want to include logo directly
            return _.map(this.allComponents, (component) => {
              const componentId = component.id;

              const qualificationStatus =
                qualificationsStore.qualificationStatusByComponentId[
                  componentId
                ];

              // TODO: probably dont need this generic status icon setup anymore...
              const statusIcons: DiagramStatusIcon[] = _.compact([
                {
                  ...qualificationStatusToIconMap[
                    qualificationStatus ?? "notexists"
                  ],
                  tabSlug: "qualifications",
                },
                component.hasResource
                  ? { icon: "check-hex", tone: "success", tabSlug: "resource" }
                  : { icon: "none" },
              ]);

              return {
                ..._.omit(component, "parentId"),
                // swapping "id" to be node id and passing along component id separately for the diagram
                // this is gross and needs to go, but will happen later
                id: component.id,
                componentId: component.id,
                parentComponentId: component.parentId,
                title: component.displayName,
                subtitle: component.schemaName,
                isLoading:
                  !!statusStore.componentStatusById[componentId]?.isUpdating,
                typeIcon: component?.icon || "logo-si",
                statusIcons,
              };
            });
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
            const validEdges = _.filter(this.allEdges, (edge) => {
              return (
                !!this.componentsById[edge.toComponentId] &&
                !!this.componentsById[edge.fromComponentId]
              );
            });

            // If edge connects inside ancestry, don't show
            return _.map(validEdges, (edge) => {
              const fromComponent = this.componentsById[edge.fromComponentId];
              if (!fromComponent)
                throw Error(`Not finding from node for edge ${edge.id}`);
              const fromParentage = [
                fromComponent.id,
                ...(fromComponent.ancestorIds ?? []),
              ];

              const toComponent = this.componentsById[edge.toComponentId];
              if (!toComponent)
                throw Error(`Not finding to node for edge ${edge.id}`);
              const toParentage = [
                toComponent.id,
                ...(toComponent.ancestorIds ?? []),
              ];

              const isInvisible =
                fromParentage.includes(toComponent.id) ||
                toParentage.includes(fromComponent.id);

              return { ...edge, isInvisible };
            });
          },

          schemas: (state) => _.values(state.schemasById),

          schemaVariants(): DiagramSchemaVariantWithDisplayMetadata[] {
            // NOTE(nick): there is likely a prettier way to do this using lodash. Sorry Wendy and John <3.
            const schemaVariants = [];
            for (const schema of this.schemas) {
              for (const variant of schema.variants) {
                schemaVariants.push({
                  id: variant.id,
                  name: variant.name,
                  builtin: variant.builtin,

                  color: variant.color,
                  category: variant.category,
                  inputSockets: variant.inputSockets,
                  outputSockets: variant.outputSockets,

                  schemaName: schema.name,
                });
              }
            }
            return schemaVariants;
          },

          schemaVariantsById(): Record<
            string,
            DiagramSchemaVariantWithDisplayMetadata
          > {
            return _.keyBy(this.schemaVariants, "id");
          },

          schemasWithAtLeastOneVariant(): DiagramSchemaWithDisplayMetadata[] {
            // NOTE(nick): there is likely a prettier way to do this using lodash. Sorry Wendy and John <3.
            const schemasWithAtLeastOneVariant = [];
            for (const schema of this.schemas) {
              if (schema.variants[0]) {
                schemasWithAtLeastOneVariant.push({
                  id: schema.id,
                  name: schema.name,
                  builtin: schema.builtin,
                  variants: schema.variants,
                  category: schema.variants[0].category,
                  color: schema.variants[0].color,
                });
              }
            }
            return schemasWithAtLeastOneVariant;
          },

          schemasByCategory(): Record<
            string,
            DiagramSchemaWithDisplayMetadata[]
          > {
            return _.groupBy(
              this.schemasWithAtLeastOneVariant,
              (s) => s.category,
            );
          },

          categories(): Categories {
            return _.map(this.schemasByCategory, (schemas, category) => ({
              displayName: category,
              schemas,
            }));
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
        },
        actions: {
          // actually fetches diagram-style data, but we have a computed getter to turn back into more generic component data above
          async FETCH_DIAGRAM_DATA() {
            return new ApiRequest<{
              components: RawComponent[];
              edges: Edge[];
            }>({
              url: "diagram/get_diagram",
              params: {
                ...visibilityParams,
              },
              onSuccess: (response) => {
                this.rawComponentsById = _.keyBy(response.components, "id");
                this.edgesById = _.keyBy(response.edges, "id");

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
                });
                // and set the selection to the new component
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
            return new ApiRequest<Array<DiagramSchema>>({
              url: "diagram/list_schemas",
              params: {
                ...visibilityParams,
              },
              onSuccess: (response) => {
                this.schemasById = _.keyBy(response, "id");
              },
            });
          },

          async SET_COMPONENT_DIAGRAM_POSITION(
            componentId: ComponentId,
            position: Vector2d,
            size?: Size2D,
          ) {
            let width;
            let height;
            if (size) {
              width = Math.round(size.width).toString();
              height = Math.round(size.height).toString();
            }

            return new ApiRequest<{ componentStats: ComponentStats }>({
              method: "post",
              url: "diagram/set_component_position",
              params: {
                componentId,
                x: Math.round(position.x).toString(),
                y: Math.round(position.y).toString(),
                width,
                height,
                diagramKind: "configuration",
                ...visibilityParams,
              },
              onSuccess: (response) => {
                // record position change rather than wait for re-fetch
              },
            });
          },

          setInsertSchema(schemaId: SchemaId) {
            this.selectedInsertSchemaId = schemaId;
            this.setSelectedComponentId(null);
          },
          cancelInsert() {
            this.selectedInsertSchemaId = null;
          },

          async CREATE_COMPONENT(
            schemaId: string,
            position: Vector2d,
            parentId?: string,
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
                schemaId,
                parentId,
                x: position.x.toString(),
                y: position.y.toString(),
                ...visibilityParams,
              },
              optimistic: () => {
                this.pendingInsertedComponents[tempInsertId] = {
                  tempId: tempInsertId,
                  position,
                };

                return () => {
                  delete this.pendingInsertedComponents[tempInsertId];
                };
              },
              onSuccess: (response) => {
                // we'll link up our temporary id to the actual ID
                // so we can hide the spinning temporary insert placeholder when the data is loaded
                const pendingInsert =
                  this.pendingInsertedComponents[tempInsertId];
                if (pendingInsert) {
                  pendingInsert.componentId = response.componentId;
                }

                // TODO: ideally here we would set the selected component id, but the component doesn't exist in the store yet
                // so we'll have to do it in the FETCH_DIAGRAM when we delete the pending insert
                // in the future, we should probably return at least basic info about the component from the create call
                // so we can select it right away and at least show a loading screen as more data is fetched
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
                  createdInfo: {
                    timestamp: nowTs,
                    actor: { kind: "user", label: "You" },
                  },
                };
                return () => {
                  delete this.edgesById[tempId];
                };
              },
            });
          },
          async CONNECT_COMPONENT_TO_FRAME(
            childId: ComponentId,
            parentId: ComponentId,
          ) {
            if (changeSetsStore.creatingChangeSet)
              throw new Error("race, wait until the change set is created");
            if (changeSetId === changeSetsStore.headChangeSetId)
              changeSetsStore.creatingChangeSet = true;

            return new ApiRequest<{ node: DiagramNode }>({
              method: "post",
              url: "diagram/connect_component_to_frame",
              params: {
                childId,
                parentId,
                ...visibilityParams,
              },
              optimistic: () => {
                const component = this.rawComponentsById[childId];
                if (!component) return;
                const prevParentId = component?.parentId;
                component.parentId = parentId;
                return () => {
                  component.parentId = prevParentId;
                };
              },
            });
          },
          async DETACH_COMPONENT(componentId: ComponentId) {
            if (changeSetsStore.creatingChangeSet)
              throw new Error("race, wait until the change set is created");
            if (changeSetId === changeSetsStore.headChangeSetId)
              changeSetsStore.creatingChangeSet = true;

            return new ApiRequest<{ node: DiagramNode }>({
              method: "post",
              url: "diagram/detach_component",
              params: {
                componentId,
                ...visibilityParams,
              },
              optimistic: () => {
                const component = this.rawComponentsById[componentId];
                if (!component) return;
                const prevParentId = component?.parentId;
                delete component.parentId;
                return () => {
                  component.parentId = prevParentId;
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

          async MIGRATE_COMPONENT(componentId: ComponentId) {
            return new ApiRequest({
              url: "component/migrate_to_default_variant",
              keyRequestStatusBy: componentId,
              method: "post",
              params: {
                componentId,
                ...visibilityParams,
              },
            });
          },

          async DELETE_EDGE(edgeId: EdgeId) {
            if (changeSetsStore.creatingChangeSet)
              throw new Error("race, wait until the change set is created");
            if (changeSetId === changeSetsStore.headChangeSetId)
              changeSetsStore.creatingChangeSet = true;

            return new ApiRequest({
              method: "post",
              url: "diagram/delete_connection",
              keyRequestStatusBy: edgeId,
              params: {
                edgeId,
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

          async RESTORE_EDGE(edgeId: EdgeId) {
            if (changeSetsStore.creatingChangeSet)
              throw new Error("race, wait until the change set is created");
            if (changeSetId === changeSetsStore.headChangeSetId)
              changeSetsStore.creatingChangeSet = true;

            return new ApiRequest({
              method: "post",
              url: "diagram/restore_connection",
              keyRequestStatusBy: edgeId,
              params: {
                edgeId,
                ...visibilityParams,
              },
              onSuccess: (response) => {
                // this.componentDiffsById[componentId] = response.componentDiff;
              },
              optimistic: () => {
                const originalEdge = this.edgesById[edgeId];
                if (originalEdge) {
                  this.edgesById[edgeId] = {
                    ...originalEdge,
                    changeStatus: "unmodified",
                    deletedInfo: undefined,
                  };
                }

                return () => {
                  if (originalEdge) {
                    this.edgesById[edgeId] = originalEdge;
                  }
                  this.selectedEdgeId = edgeId;
                };
              },
            });
          },

          async DELETE_COMPONENT(componentId: ComponentId) {
            if (changeSetsStore.creatingChangeSet)
              throw new Error("race, wait until the change set is created");
            if (changeSetId === changeSetsStore.headChangeSetId)
              changeSetsStore.creatingChangeSet = true;

            return new ApiRequest({
              method: "post",
              url: "diagram/delete_component",
              keyRequestStatusBy: componentId,
              params: {
                componentId,
                ...visibilityParams,
              },
              onSuccess: (response) => {
                // this.componentDiffsById[componentId] = response.componentDiff;
              },
              optimistic: () => {
                const component = this.rawComponentsById[componentId];
                const originalStatus = component?.changeStatus;
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

                return () => {
                  if (component && originalStatus) {
                    this.rawComponentsById[componentId] = {
                      ...component,
                      toDelete: true,
                      changeStatus: originalStatus,
                      deletedInfo: undefined,
                    };
                  }
                };
              },
            });
          },

          async PASTE_COMPONENTS(
            componentIds: ComponentId[],
            offset: { x: number; y: number },
            position: { x: number; y: number },
            newParentNodeId?: ComponentNodeId,
          ) {
            if (changeSetsStore.creatingChangeSet)
              throw new Error("race, wait until the change set is created");
            if (changeSetId === changeSetsStore.headChangeSetId)
              changeSetsStore.creatingChangeSet = true;

            this.pastingId = null;
            this.pastingLoading = true;
            this.pastingError = undefined;

            return new ApiRequest<{
              id: string;
            }>({
              method: "post",
              url: "diagram/paste_components",
              keyRequestStatusBy: componentIds,
              params: {
                componentIds,
                offsetX: offset.x,
                offsetY: offset.y,
                newParentNodeId,
                ...visibilityParams,
              },
              onSuccess: (data) => {
                this.pastingId = data.id;
                this.pendingInsertedComponents[this.pastingId] = {
                  tempId: this.pastingId,
                  position,
                };
              },
              onFail: () => {
                this.pastingId = null;
                this.pastingLoading = false;
              },
            });
          },

          async DELETE_COMPONENTS(componentIds: ComponentId[]) {
            if (changeSetsStore.creatingChangeSet)
              throw new Error("race, wait until the change set is created");
            if (changeSetId === changeSetsStore.headChangeSetId)
              changeSetsStore.creatingChangeSet = true;

            return new ApiRequest({
              method: "post",
              url: "diagram/delete_components",
              keyRequestStatusBy: componentIds,
              params: {
                componentIds,
                ...visibilityParams,
              },
              optimistic: () => {
                for (const componentId of componentIds) {
                  const component = this.rawComponentsById[componentId];
                  if (component) {
                    this.rawComponentsById[componentId] = {
                      ...component,
                      changeStatus: "deleted",
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
                        deletedInfo: undefined,
                      };
                    }
                  }
                };
              },
            });
          },

          async RESTORE_COMPONENTS(componentIds: ComponentId[]) {
            if (changeSetsStore.creatingChangeSet)
              throw new Error("race, wait until the change set is created");
            if (changeSetId === changeSetsStore.headChangeSetId)
              changeSetsStore.creatingChangeSet = true;

            return new ApiRequest({
              method: "post",
              url: "diagram/remove_delete_intent",
              keyRequestStatusBy: componentIds,
              params: {
                componentIds,
                ...visibilityParams,
              },
              onSuccess: (response) => {
                // this.componentDiffsById[componentId] = response.componentDiff;
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
            this.selectedEdgeId = null;
            if (!selection || !selection.length) {
              this.selectedComponentIds = [];
              // forget which details tab is active when selection is cleared
              this.selectedComponentDetailsTab = null;
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

          async REFRESH_ALL_RESOURCE_INFO() {
            return new ApiRequest({
              method: "post",
              url: "component/refresh",
              params: {
                workspaceId: visibilityParams.workspaceId,
                visibility_change_set_pk: changeSetsStore.headChangeSetId,
              },
              onSuccess: (response) => {
                // do nothing
              },
            });
          },
        },
        onActivated() {
          if (!changeSetId) return;

          // trigger initial load
          this.FETCH_DIAGRAM_DATA();
          this.FETCH_AVAILABLE_SCHEMAS();

          // TODO: prob want to take loading state into consideration as this will set it before its loaded
          const stopWatchingUrl = watch(
            router.currentRoute,
            this.syncUrlIntoSelection,
            {
              immediate: true,
            },
          );

          // realtime subs
          const realtimeStore = useRealtimeStore();

          realtimeStore.subscribe(this.$id, `changeset/${changeSetId}`, [
            {
              eventType: "ComponentCreated",
              debounce: true,
              callback: (data) => {
                // If the component that updated wasn't in this change set,
                // don't update
                if (data.changeSetId !== changeSetId) return;
                this.FETCH_DIAGRAM_DATA();
              },
            },
            {
              eventType: "ResourceRefreshed",
              callback: (resourceRefreshedEvent) => {
                if (resourceRefreshedEvent?.componentId) {
                  this.refreshingStatus[resourceRefreshedEvent.componentId] =
                    false;
                }
              },
            },
            {
              eventType: "AsyncFinish",
              callback: ({ id }: { id: string }) => {
                if (id === this.pastingId) {
                  this.pastingLoading = false;
                  this.pastingError = undefined;
                  this.pastingId = null;
                  delete this.pendingInsertedComponents[id];
                }
              },
            },
            {
              eventType: "AsyncError",
              callback: ({ id, error }: { id: string; error: string }) => {
                if (id === this.pastingId) {
                  this.pastingLoading = false;
                  this.pastingError = error;
                  this.pastingId = null;
                  delete this.pendingInsertedComponents[id];
                }
              },
            },
          ]);

          return () => {
            // clear selection without triggering url stuff
            this.selectedComponentIds = [];
            this.selectedEdgeId = null;

            stopWatchingUrl();
            realtimeStore.unsubscribe(this.$id);
          };
        },
      },
    ),
  )();
};
