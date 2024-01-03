import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { Vector2d } from "konva/lib/types";
import { ApiRequest, addStoreHooks } from "@si/vue-lib/pinia";
import { IconNames } from "@si/vue-lib/design-system";

import mitt from "mitt";
import { watch } from "vue";
import {
  DiagramEdgeDef,
  DiagramNodeDef,
  DiagramSocketDef,
  DiagramStatusIcon,
  GridPoint,
  Size2D,
} from "@/components/ModelingDiagram/diagram_types";
import {
  DiagramNode,
  DiagramSchemaVariant,
  DiagramSchemaVariants,
} from "@/api/sdf/dal/diagram";
import { ComponentStats, ChangeStatus } from "@/api/sdf/dal/change_set";
import { ComponentDiff } from "@/api/sdf/dal/component";
import { Resource } from "@/api/sdf/dal/resource";
import { CodeView } from "@/api/sdf/dal/code_view";
import { ActorView } from "@/api/sdf/dal/history_actor";
import { nilId } from "@/utils/nilId";
import router from "@/router";
import { ChangeSetId, useChangeSetsStore } from "./change_sets.store";
import { useRealtimeStore } from "./realtime/realtime.store";
import {
  QualificationStatus,
  useQualificationsStore,
} from "./qualifications.store";
import { useWorkspacesStore } from "./workspaces.store";
import { useStatusStore } from "./status.store";

export type ComponentId = string;
export type EdgeId = string;
export type ProviderId = string;

type SchemaVariantId = string;

type RawComponent = {
  changeStatus: ChangeStatus;
  childComponentIds: ComponentId[];
  color: string;
  createdInfo: ActorAndTimestamp;
  deletedInfo?: ActorAndTimestamp;
  displayName: string;
  hasResource: boolean;
  id: ComponentId;
  nodeType: "component" | "configurationFrame" | "aggregationFrame";
  parentComponentId?: ComponentId;
  position: GridPoint;
  schemaCategory: string;
  schemaId: string; // TODO: probably want to move this to a different store and not load it all the time
  schemaName: string;
  schemaVariantId: string;
  schemaVariantName: string;
  size?: Size2D;
  sockets: DiagramSocketDef[];
  updatedInfo: ActorAndTimestamp;
};

export type FullComponent = RawComponent & {
  // direct parent ID
  parentId?: ComponentId;
  // array of parent IDs
  ancestorIds?: ComponentId[];
  childIds?: ComponentId[];
  matchesFilter: boolean;
  icon: IconNames;
  isGroup: false;
};

type Edge = {
  id: EdgeId;
  fromComponentId: ComponentId;
  fromExternalProviderId: ProviderId;
  toComponentId: ComponentId;
  toExplicitInternalProviderId: ProviderId;
  isInvisible?: boolean;
  /** change status of edge in relation to head */
  changeStatus?: ChangeStatus;
  createdInfo: ActorAndTimestamp;
  // updatedInfo?: ActorAndTimestamp; // currently we dont ever update an edge...
  deletedInfo?: ActorAndTimestamp;
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

export interface AttributeDebugData {
  valueId: string;
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
  prototypeContext: {
    prop_id: string;
    internal_provider_id: string;
    external_provider_id: string;
    component_id: string;
  };
  kind: string;
  prototypeInChangeSet: boolean;
  valueInChangeSet: boolean;
  implicitValue?: object | string | number | boolean | null;
  implicitValueContext?: {
    prop_id: string;
    internal_provider_id: string;
    external_provider_id: string;
    component_id: string;
  };
  implicitFuncName?: string;
}

export interface AttributeDebugView {
  path: string;
  name: string;
  debugData: AttributeDebugData;
}

export interface ComponentDebugView {
  name: string;
  schemaVariantId: string;
  attributes: AttributeDebugView[];
  inputSockets: AttributeDebugView[];
  outputSockets: AttributeDebugView[];
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
          schemaVariantsById: {} as Record<
            SchemaVariantId,
            DiagramSchemaVariant
          >,

          copyingFrom: null as { x: number; y: number } | null,
          selectedComponentIds: [] as ComponentId[],
          selectedEdgeId: null as EdgeId | null,
          selectedComponentDetailsTab: null as string | null,
          hoveredComponentId: null as ComponentId | null,
          hoveredEdgeId: null as EdgeId | null,

          panTargetComponentId: null as ComponentId | null,

          // used by the diagram to track which schema variant is selected for insertion
          selectedInsertSchemaVariantId: null as SchemaVariantId | null,

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
          componentsById(): Record<ComponentId, FullComponent> {
            const nodeIdToComponentId = _.mapValues(
              _.keyBy(this.rawComponentsById, (c) => c.id),
              (c) => c.id,
            );

            const getAncestorIds = (
              componentId: ComponentId,
              idsArray = [] as ComponentId[],
            ): ComponentId[] => {
              const c = this.rawComponentsById[componentId];

              if (!c) throw new Error("what?");
              const parentId = c.parentComponentId;

              if (parentId) {
                return getAncestorIds(parentId, [parentId, ...idsArray]);
              } else {
                return idsArray;
              }
            };

            return _.mapValues(this.rawComponentsById, (rc) => {
              // these categories should probably have a name and a different displayName (ie "aws" vs "Amazon AWS")
              // and eventually can just assume the icon is `logo-${name}`
              const typeIcon =
                {
                  AWS: "logo-aws",
                  "AWS EC2": "logo-aws",
                  CoreOS: "logo-coreos",
                  Docker: "logo-docker",
                  Kubernetes: "logo-k8s",
                }[rc?.schemaCategory || ""] || "logo-si"; // fallback to SI logo

              const ancestorIds = getAncestorIds(rc.id);

              return {
                ...rc,
                ancestorIds,
                parentId: _.last(ancestorIds),
                childIds: _.map(
                  rc.childComponentIds,
                  (nodeId) => nodeIdToComponentId[nodeId],
                ),
                icon: typeIcon,
                isGroup: rc.nodeType !== "component",
              } as FullComponent;
            });
          },
          componentsByParentId(): Record<ComponentId, FullComponent[]> {
            return _.groupBy(this.allComponents, (c) => c.parentComponentId);
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
                ...component,
                id: component.id,
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
            // Note(victor): The code below checks whether was only created implicitly, through inheritance from an aggregation frame
            // In the future, it would make more sense for these edges to not be returned from the backend
            const validEdges = _.filter(this.allEdges, (edge) => {
              return (
                !!this.componentsById[edge.toComponentId] &&
                !!this.componentsById[edge.fromComponentId]
              );
            });
            const edgesWithInvisibleSet = _.map(validEdges, (rawEdge) => {
              const edge = { ...rawEdge, invisible: false };

              const toComponentParentId =
                this.componentsById[edge.toComponentId]?.parentComponentId;

              if (toComponentParentId) {
                const toComponentParent =
                  this.componentsById[toComponentParentId];

                if (toComponentParent?.nodeType === "aggregationFrame") {
                  if (edge.fromComponentId === toComponentParent.id) {
                    edge.isInvisible = true;
                  }
                }
              }

              const fromComponentParentId =
                this.componentsById[edge.fromComponentId]?.parentComponentId;

              if (fromComponentParentId) {
                const fromParentComp =
                  this.componentsById[fromComponentParentId];
                if (fromParentComp?.nodeType === "aggregationFrame") {
                  if (edge.toComponentId === fromParentComp.id) {
                    edge.isInvisible = true;
                  }
                }
              }

              return edge;
            });

            return edgesWithInvisibleSet;
          },

          schemaVariants: (state) => _.values(state.schemaVariantsById),

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
            _.each(_.values(state.edgesById), (edge) => {
              const fromComponentId = edge.fromComponentId;
              const toComponentId = edge.toComponentId;
              connectedComponents[fromComponentId] ||= [];
              connectedComponents[fromComponentId]!.push(toComponentId); // eslint-disable-line @typescript-eslint/no-non-null-assertion
            });

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
          // TODO: change these endpoints to return a more complete picture of component data in one call
          // see also component/get_components_metadata endpoint which was not used anymore but has some more data we may want to include

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

          // used when adding new nodes
          async FETCH_AVAILABLE_SCHEMAS() {
            return new ApiRequest<DiagramSchemaVariants>({
              // TODO: probably switch to something like GET `/workspaces/:id/schemas`?
              url: "diagram/list_schema_variants",
              params: {
                ...visibilityParams,
              },
              onSuccess: (response) => {
                this.schemaVariantsById = _.keyBy(response, "id");
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
                ...visibilityParams,
              },
              onSuccess: (response) => {
                // record position change rather than wait for re-fetch
              },
            });
          },

          setInsertSchemaVariant(schemaVariantId: SchemaVariantId) {
            this.selectedInsertSchemaVariantId = schemaVariantId;
            this.setSelectedComponentId(null);
          },
          cancelInsert() {
            this.selectedInsertSchemaVariantId = null;
          },

          async CREATE_COMPONENT(
            schemaVariantId: string,
            position: Vector2d,
            parentComponentId?: string,
          ) {
            if (changeSetsStore.creatingChangeSet)
              throw new Error("race, wait until the change set is created");
            if (changeSetId === nilId())
              changeSetsStore.creatingChangeSet = true;

            const tempInsertId = _.uniqueId("temp-insert-component");

            return new ApiRequest<{
              componentId: ComponentId;
            }>({
              method: "post",
              url: "diagram/create_component",
              headers: { accept: "application/json" },
              params: {
                schemaVariantId,
                parentId: parentComponentId,
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
            from: { componentId: ComponentId; externalProviderId: ProviderId },
            to: {
              componentId: ComponentId;
              explicitInternalProviderId: ProviderId;
            },
          ) {
            if (changeSetsStore.creatingChangeSet)
              throw new Error("race, wait until the change set is created");
            if (changeSetId === nilId())
              changeSetsStore.creatingChangeSet = true;

            const tempId = `temp-edge-${+new Date()}`;

            return new ApiRequest<{
              connection: {
                id: string;
              };
              forceChangesetPk?: string;
            }>({
              method: "post",
              url: "diagram/create_connection",
              params: {
                fromComponentId: from.componentId,
                fromExternalProviderId: from.externalProviderId,
                toComponentId: to.componentId,
                toExplicitInternalProviderId: to.explicitInternalProviderId,
                ...visibilityParams,
              },
              onSuccess: (response) => {
                // change our temporary id to the real one, only if we haven't re-fetched the diagram yet
                if (this.edgesById[tempId]) {
                  const edge = this.edgesById[tempId];
                  if (edge) {
                    this.edgesById[response.connection.id] = edge;
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
                  fromExternalProviderId: from.externalProviderId,
                  toComponentId: to.componentId,
                  toExplicitInternalProviderId: to.explicitInternalProviderId,
                  changeStatus: "added",
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
            if (changeSetId === nilId())
              changeSetsStore.creatingChangeSet = true;

            return new ApiRequest<{ node: DiagramNode }>({
              method: "post",
              url: "diagram/connect_component_to_frame",
              params: {
                childId,
                parentId,
                ...visibilityParams,
              },
              onSuccess: (response) => {
                // TODO: store component details rather than waiting for re-fetch
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

          async DELETE_EDGE(edgeId: EdgeId) {
            if (changeSetsStore.creatingChangeSet)
              throw new Error("race, wait until the change set is created");
            if (changeSetId === nilId())
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
            if (changeSetId === nilId())
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
            if (changeSetId === nilId())
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
                    deletedInfo: {
                      timestamp: new Date().toISOString(),
                      actor: { kind: "user", label: "You" },
                    },
                  };
                }

                // TODO: optimistically delete connected edges?
                // not super important...
                return () => {
                  if (component && originalStatus) {
                    this.rawComponentsById[componentId] = {
                      ...component,
                      changeStatus: originalStatus,
                      deletedInfo: undefined,
                    };
                  }
                };
              },
            });
          },
          async RESTORE_COMPONENT(componentId: ComponentId) {
            if (changeSetsStore.creatingChangeSet)
              throw new Error("race, wait until the change set is created");
            if (changeSetId === nilId())
              changeSetsStore.creatingChangeSet = true;

            return new ApiRequest({
              method: "post",
              url: "diagram/restore_component",
              keyRequestStatusBy: componentId,
              params: {
                componentId,
                ...visibilityParams,
              },
              onSuccess: (response) => {
                // this.componentDiffsById[componentId] = response.componentDiff;
              },
            });
          },

          async PASTE_COMPONENTS(
            componentIds: ComponentId[],
            offset: { x: number; y: number },
          ) {
            if (changeSetsStore.creatingChangeSet)
              throw new Error("race, wait until the change set is created");
            if (changeSetId === nilId())
              changeSetsStore.creatingChangeSet = true;

            return new ApiRequest({
              method: "post",
              url: "diagram/paste_components",
              keyRequestStatusBy: componentIds,
              params: {
                componentIds,
                offsetX: offset.x,
                offsetY: offset.y,
                ...visibilityParams,
              },
              onSuccess: (response) => {},
            });
          },

          async DELETE_COMPONENTS(componentIds: ComponentId[]) {
            if (changeSetsStore.creatingChangeSet)
              throw new Error("race, wait until the change set is created");
            if (changeSetId === nilId())
              changeSetsStore.creatingChangeSet = true;

            return new ApiRequest({
              method: "post",
              url: "diagram/delete_components",
              keyRequestStatusBy: componentIds,
              params: {
                componentIds,
                ...visibilityParams,
              },
              onSuccess: (response) => {
                // this.componentDiffsById[componentId] = response.componentDiff;
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
            if (changeSetId === nilId())
              changeSetsStore.creatingChangeSet = true;

            return new ApiRequest({
              method: "post",
              url: "diagram/restore_components",
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

            router.replace({
              // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
              name: router.currentRoute.value.name!,
              query: {
                ...(selectedIds.length && { s: selectedIds.join("|") }),
                ...(this.selectedComponentDetailsTab && {
                  t: this.selectedComponentDetailsTab,
                }),
              },
            });
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
          setHoveredComponentId(id: ComponentId | null) {
            this.hoveredComponentId = id;
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
                visibility_change_set_pk: nilId(),
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
                visibility_change_set_pk: nilId(),
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
              callback: (_update) => {
                this.FETCH_DIAGRAM_DATA();
              },
            },
            {
              eventType: "ChangeSetWritten",
              callback: (writtenChangeSetId) => {
                // ideally we wouldn't have to check this - since the topic subscription
                // would mean we only receive the event for this changeset already...
                // but this is fine for now
                if (writtenChangeSetId !== changeSetId) return;

                // probably want to get pushed updates instead of blindly re-fetching, but this is the first step of getting things working
                this.FETCH_DIAGRAM_DATA();
              },
            },
            {
              eventType: "CodeGenerated",
              callback: (codeGeneratedEvent) => {
                this.FETCH_COMPONENT_CODE(codeGeneratedEvent.componentId);
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
          ]);

          return () => {
            stopWatchingUrl();
            realtimeStore.unsubscribe(this.$id);
          };
        },
      },
    ),
  )();
};
