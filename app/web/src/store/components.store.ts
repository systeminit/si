import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { Vector2d } from "konva/lib/types";
import { ApiRequest, addStoreHooks } from "@si/vue-lib/pinia";
import { IconNames } from "@si/vue-lib/design-system";

import {
  DiagramEdgeDef,
  DiagramNodeDef,
  DiagramSocketDef,
  DiagramStatusIcon,
  GridPoint,
  Size2D,
} from "@/components/GenericDiagram/diagram_types";
import { MenuItem } from "@/api/sdf/dal/menu";
import {
  DiagramNode,
  DiagramSchemaVariant,
  DiagramSchemaVariants,
} from "@/api/sdf/dal/diagram";
import {
  ComponentStats,
  ChangeStatus,
  ActionPrototype,
} from "@/api/sdf/dal/change_set";
import { ComponentDiff } from "@/api/sdf/dal/component";
import { Resource } from "@/api/sdf/dal/resource";
import { CodeView } from "@/api/sdf/dal/code_view";
import { ActorView } from "@/api/sdf/dal/history_actor";
import { nilId } from "@/utils/nilId";
import { ChangeSetId, useChangeSetsStore } from "./change_sets.store";
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
type SchemaVariantId = string;

type RawComponent = {
  id: ComponentId;
  nodeId: ComponentNodeId;
  displayName: string;
  parentNodeId?: ComponentNodeId;
  childNodeIds: ComponentNodeId[];
  schemaName: string;
  schemaId: string;
  schemaVariantId: string;
  schemaVariantName: string;
  schemaCategory: string;
  color: string;
  nodeType: "component" | "configurationFrame" | "aggregationFrame";
  position: GridPoint;
  changeStatus: ChangeStatus;
  actions: ActionPrototype[];
  resource: Resource; // TODO: probably want to move this to a different store and not load it all the time
  sockets: DiagramSocketDef[];
  createdInfo: ActorAndTimestamp;
  updatedInfo: ActorAndTimestamp;
  deletedInfo?: ActorAndTimestamp;
};

export type FullComponent = RawComponent & {
  parentNodeId?: ComponentNodeId;
  parentId?: ComponentId;
  childNodeIds?: ComponentNodeId[];
  childIds?: ComponentId[];
  isGroup: boolean;
  matchesFilter: boolean;
  icon: IconNames;
};

type Edge = {
  id: EdgeId;
  fromNodeId: ComponentNodeId;
  fromSocketId: SocketId;
  toNodeId: ComponentNodeId;
  toSocketId: SocketId;
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

export type MenuSchema = {
  id: SchemaId;
  displayName: string;
  color: string;
};

type NodeAddMenu = {
  displayName: string;
  schemas: MenuSchema[];
}[];

const qualificationStatusToIconMap: Record<
  QualificationStatus,
  DiagramStatusIcon
> = {
  success: { icon: "check-circle", tone: "success" },
  warning: { icon: "exclamation-circle", tone: "warning" },
  failure: { icon: "x-circle", tone: "error" },
  running: { icon: "loader", tone: "info" },
};

const confirmationStatusToIconMap: Record<
  "success" | "failure",
  DiagramStatusIcon
> = {
  success: { icon: "check-circle", tone: "success" },
  failure: { icon: "tools", tone: "error" },
};

export const useComponentsStore = (forceChangeSetId?: ChangeSetId) => {
  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspacePk;

  const changeSetsStore = useChangeSetsStore();

  // this needs some work... but we'll probably want a way to force using HEAD
  // so we can load HEAD data in some scenarios while also loading a change set?
  let changeSetId: ChangeSetId | null;
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
    defineStore(`cs${changeSetId || "NONE"}/components`, {
      state: () => ({
        // components within this changeset
        // componentsById: {} as Record<ComponentId, Component>,
        // connectionsById: {} as Record<ConnectionId, Connection>,

        componentCodeViewsById: {} as Record<ComponentId, CodeView[]>,
        componentDiffsById: {} as Record<ComponentId, ComponentDiff>,

        rawComponentsById: {} as Record<ComponentId, RawComponent>,

        edgesById: {} as Record<EdgeId, Edge>,
        schemaVariantsById: {} as Record<SchemaVariantId, DiagramSchemaVariant>,
        rawNodeAddMenu: [] as MenuItem[],

        selectedComponentIds: [] as ComponentId[],
        selectedEdgeId: null as EdgeId | null,
        hoveredComponentId: null as ComponentId | null,
        hoveredEdgeId: null as EdgeId | null,

        panTargetComponentId: null as ComponentId | null,

        // used by the diagram to track which schema is selected for insertion
        selectedInsertSchemaId: null as SchemaId | null,

        refreshingStatus: {} as Record<ComponentId, boolean>,
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
            _.keyBy(this.rawComponentsById, (c) => c.nodeId),
            (c) => c.id,
          );
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

            return {
              ...rc,
              // convert "node" ids back to component ids, so we can use that in a few places
              parentId: rc.parentNodeId
                ? nodeIdToComponentId[rc.parentNodeId]
                : undefined,
              childIds: _.map(
                rc.childNodeIds,
                (nodeId) => nodeIdToComponentId[nodeId],
              ),
              icon: typeIcon,
              isGroup: rc.nodeType !== "component",
            } as FullComponent;
          });
        },
        componentsByParentId(): Record<ComponentId, FullComponent[]> {
          return _.groupBy(this.allComponents, (c) =>
            // remapping to component id... PLEASE LETS KILL NODE ID!
            c.parentNodeId
              ? this.componentsByNodeId[c.parentNodeId]?.id
              : "root",
          );
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

        componentsByNodeId(): Record<ComponentNodeId, FullComponent> {
          return _.keyBy(_.values(this.componentsById), (c) => c.nodeId);
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
        selectedComponentDiff(): ComponentDiff | undefined {
          return this.componentDiffsById[this.selectedComponentId || 0];
        },
        selectedComponentCode(): CodeView[] | undefined {
          return this.componentCodeViewsById[this.selectedComponentId || 0];
        },

        diagramNodes(): DiagramNodeDef[] {
          const qualificationsStore = useQualificationsStore();
          const statusStore = useStatusStore();

          // adding logo and qualification info into the nodes
          // TODO: probably want to include logo directly
          return _.map(this.allComponents, (component) => {
            const componentId = component.id;

            const qualificationStatus =
              qualificationsStore.qualificationStatusByComponentId[componentId];
            const confirmationStatus =
              changeSetsStore.statusByComponentId[componentId];

            const statusIcons: DiagramStatusIcon[] = _.compact([
              confirmationStatus
                ? confirmationStatusToIconMap[confirmationStatus]
                : {
                    icon: "minus",
                    tone: "neutral",
                  },
              qualificationStatusToIconMap[qualificationStatus ?? "failure"],
            ]);

            return {
              ...component,
              // swapping "id" to be node id and passing along component id separately for the diagram
              // this is gross and needs to go, but will happen later
              id: component.nodeId,
              componentId: component.id,
              title: component.displayName,
              subtitle: component.schemaName,
              isLoading:
                !!statusStore.componentStatusById[componentId]?.isUpdating,
              typeIcon: component?.icon || "logo-si",
              statusIcons,
            };
          });
        },

        diagramEdges(): DiagramEdgeDef[] {
          return this.allEdges;
        },

        edgesByFromNodeId(): Record<ComponentNodeId, Edge[]> {
          return _.groupBy(this.allEdges, (e) => e.fromNodeId);
        },

        edgesByToNodeId(): Record<ComponentNodeId, Edge[]> {
          return _.groupBy(this.allEdges, (e) => e.toNodeId);
        },

        schemaVariants: (state) => _.values(state.schemaVariantsById),

        nodeAddMenu(): NodeAddMenu {
          return _.compact(
            _.map(this.rawNodeAddMenu, (category) => {
              // all root level items are categories for now... will probably rework this endpoint anyway
              if (category.kind !== "category") return null;
              return {
                displayName: category.name,
                // TODO: add color + logo on categories?
                schemas: _.compact(
                  _.map(category.items, (item) => {
                    // ignoring "link" items - don't think these are relevant at the moment
                    if (item.kind !== "item") return;

                    // TODO: return hex code from backend...
                    const schemaVariant = Object.values(
                      this.schemaVariantsById,
                    ).find((v) => v.schemaId === item.schema_id);
                    return {
                      displayName: item.name,
                      id: item.schema_id,
                      // links: item.links, // not sure this is needed?
                      color: schemaVariant?.color ?? "#777",
                    };
                  }),
                ),
              };
            }),
          );
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
          // TODO: this is ugly... much of this logic is duplicated in GenericDiagram

          const connectedNodes: Record<ComponentId, ComponentId[]> = {};
          _.each(_.values(state.edgesById), (edge) => {
            const fromNodeId = edge.fromNodeId;
            const toNodeId = edge.toNodeId;
            connectedNodes[fromNodeId] ||= [];
            connectedNodes[fromNodeId]!.push(toNodeId); // eslint-disable-line @typescript-eslint/no-non-null-assertion
          });

          const connectedIds: ComponentId[] = [componentId];

          function walkGraph(id: ComponentId) {
            const nextIds = connectedNodes[id];
            nextIds?.forEach((nid) => {
              if (connectedIds.includes(nid)) return;
              connectedIds.push(nid);
              walkGraph(nid);
            });
          }

          walkGraph(componentId);

          return connectedIds;
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

        async FETCH_NODE_ADD_MENU() {
          return new ApiRequest<MenuItem[]>({
            method: "post",
            // TODO: probably combine into single call with FETCH_AVAILABLE_SCHEMAS
            url: "diagram/get_node_add_menu",
            params: {
              ...visibilityParams,
            },
            onSuccess: (response) => {
              this.rawNodeAddMenu = response;
            },
          });
        },

        async SET_COMPONENT_DIAGRAM_POSITION(
          nodeId: ComponentNodeId,
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
            url: "diagram/set_node_position",
            params: {
              nodeId,
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
        async CREATE_COMPONENT(
          schemaId: string,
          position: Vector2d,
          parentNodeId?: string,
        ) {
          if (changeSetsStore.creatingChangeSet)
            throw new Error("race, wait until the change set is created");
          if (changeSetId === nilId()) changeSetsStore.creatingChangeSet = true;

          return new ApiRequest<{
            componentId: ComponentId;
            nodeId: ComponentNodeId;
          }>({
            method: "post",
            url: "diagram/create_node",
            params: {
              schemaId,
              parentId: parentNodeId,
              x: position.x.toString(),
              y: position.y.toString(),
              ...visibilityParams,
            },
            onSuccess: (response) => {
              // TODO: store component details rather than waiting for re-fetch
            },
          });
        },
        async CREATE_COMPONENT_CONNECTION(
          from: { nodeId: ComponentNodeId; socketId: SocketId },
          to: { nodeId: ComponentNodeId; socketId: SocketId },
        ) {
          if (changeSetsStore.creatingChangeSet)
            throw new Error("race, wait until the change set is created");
          if (changeSetId === nilId()) changeSetsStore.creatingChangeSet = true;

          const tempId = `temp-edge-${+new Date()}`;

          return new ApiRequest<{
            connection: {
              id: string;
              classification: "configuration";
              destination: { nodeId: string; socketId: string };
              source: { nodeId: string; socketId: string };
            };
            forceChangesetPk?: string;
          }>({
            method: "post",
            url: "diagram/create_connection",
            params: {
              fromNodeId: from.nodeId,
              fromSocketId: from.socketId,
              toNodeId: to.nodeId,
              toSocketId: to.socketId,
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
                fromNodeId: from.nodeId,
                fromSocketId: from.socketId,
                toNodeId: to.nodeId,
                toSocketId: to.socketId,
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
          childNodeId: ComponentNodeId,
          parentNodeId: ComponentNodeId,
        ) {
          if (changeSetsStore.creatingChangeSet)
            throw new Error("race, wait until the change set is created");
          if (changeSetId === nilId()) changeSetsStore.creatingChangeSet = true;

          return new ApiRequest<{ node: DiagramNode }>({
            method: "post",
            url: "diagram/connect_component_to_frame",
            params: {
              childNodeId,
              parentNodeId,
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

        async DELETE_EDGE(edgeId: EdgeId) {
          if (changeSetsStore.creatingChangeSet)
            throw new Error("race, wait until the change set is created");
          if (changeSetId === nilId()) changeSetsStore.creatingChangeSet = true;

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
          if (changeSetId === nilId()) changeSetsStore.creatingChangeSet = true;

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
          if (changeSetId === nilId()) changeSetsStore.creatingChangeSet = true;

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
          if (changeSetId === nilId()) changeSetsStore.creatingChangeSet = true;

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

        async DELETE_COMPONENTS(componentIds: ComponentId[]) {
          if (changeSetsStore.creatingChangeSet)
            throw new Error("race, wait until the change set is created");
          if (changeSetId === nilId()) changeSetsStore.creatingChangeSet = true;

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
          if (changeSetId === nilId()) changeSetsStore.creatingChangeSet = true;

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

        setSelectedEdgeId(selection: EdgeId | null) {
          // clear component selection
          this.selectedComponentIds = [];
          this.selectedEdgeId = selection;
        },
        setSelectedComponentId(
          selection: ComponentId | ComponentId[] | null,
          toggleMode = false,
        ) {
          this.selectedEdgeId = null;
          if (!selection || !selection.length) {
            this.selectedComponentIds = [];
            return;
          }
          const validSelectionArray = _.reject(
            _.isArray(selection) ? selection : [selection],
            (id) => !this.componentsById[id],
          );
          if (toggleMode) {
            this.selectedComponentIds = _.xor(
              this.selectedComponentIds,
              validSelectionArray,
            );
          } else {
            this.selectedComponentIds = validSelectionArray;
          }
        },
        setHoveredComponentId(id: ComponentId | null) {
          this.hoveredComponentId = id;
          this.hoveredEdgeId = null;
        },
        setHoveredEdgeId(id: ComponentId | null) {
          this.hoveredComponentId = null;
          this.hoveredEdgeId = id;
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

        this.FETCH_DIAGRAM_DATA();
        this.FETCH_AVAILABLE_SCHEMAS();
        this.FETCH_NODE_ADD_MENU();

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
          realtimeStore.unsubscribe(this.$id);
        };
      },
    }),
  )();
};
