import { defineStore } from "pinia";
import _ from "lodash";
import { Vector2d } from "konva/lib/types";
import { ApiRequest } from "@/utils/pinia_api_tools";

import { addStoreHooks } from "@/utils/pinia_hooks_plugin";
import {
  DiagramContent,
  DiagramEdgeDef,
  DiagramNodeDef,
  DiagramStatusIcon,
} from "@/organisms/GenericDiagram/diagram_types";
import { MenuItem } from "@/api/sdf/dal/menu";
import {
  DiagramNode,
  DiagramSchemaVariant,
  DiagramSchemaVariants,
} from "@/api/sdf/dal/diagram";
import {
  ComponentStats,
  ComponentChangeStatus,
} from "@/api/sdf/dal/change_set";
import { LabelList } from "@/api/sdf/dal/label_list";
import {
  ComponentDiff,
  ComponentIdentification,
} from "@/api/sdf/dal/component";
import { Resource } from "@/api/sdf/dal/resource";
import { CodeView } from "@/api/sdf/dal/code_view";
import { IconNames } from "@/ui-lib/icons/icon_set";
import { ChangeSetId, useChangeSetsStore } from "./change_sets.store";
import { useRealtimeStore } from "./realtime/realtime.store";
import {
  QualificationStatus,
  useQualificationsStore,
} from "./qualifications.store";
import { useWorkspacesStore } from "./workspaces.store";
import { useFixesStore } from "./fixes/fixes.store";
import { useStatusStore } from "./status.store";

export type ComponentId = number;
type Component = {
  id: ComponentId;
  isGroup: boolean;
  displayName: string;
  schemaName: string;
  schemaId: number;
  schemaVariantId: number;
  schemaVariantName: string;
  icon: IconNames;
  color: string;
  changeStatus?: ComponentChangeStatus;
  // TODO: probably want to move this to a different store and not load it all the time
  resource: Resource;
};

type SocketId = number;

type SchemaId = number;

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
  failure: { icon: "x-circle", tone: "error" },
  running: { icon: "loader", tone: "info" },
};

const confirmationStatusToIconMap: Record<
  "success" | "failure" | "running",
  DiagramStatusIcon
> = {
  success: { icon: "check-square", tone: "success" },
  failure: { icon: "x-square", tone: "error" },
  running: { icon: "loader", tone: "info" },
};

const changeStatusToIconMap: Record<ComponentChangeStatus, DiagramStatusIcon> =
  {
    added: { icon: "plus-circle", tone: "success" },
    deleted: { icon: "minus-circle", tone: "error" },
    modified: { icon: "tilde-circle", tone: "warning" },
  };

export const useComponentsStore = (forceChangeSetId?: ChangeSetId) => {
  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspaceId;

  // this needs some work... but we'll probably want a way to force using HEAD
  // so we can load HEAD data in some scenarios while also loading a change set?
  let changeSetId: ChangeSetId | null;
  if (forceChangeSetId) {
    changeSetId = forceChangeSetId;
  } else {
    const changeSetsStore = useChangeSetsStore();
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
        componentIdentificationsById: {} as Record<
          ComponentId,
          ComponentIdentification
        >,
        componentChangeStatusById: {} as Record<
          ComponentId,
          ComponentChangeStatus
        >,

        componentCodeViewsById: {} as Record<ComponentId, CodeView[]>,
        componentDiffsById: {} as Record<ComponentId, ComponentDiff>,

        rawDiagramNodes: [] as DiagramNodeDef[],
        diagramEdges: [] as DiagramEdgeDef[],
        schemaVariantsById: {} as Record<SchemaId, DiagramSchemaVariant>,
        rawNodeAddMenu: [] as MenuItem[],

        selectedComponentId: null as ComponentId | null,
        lastSelectedComponentId: null as ComponentId | null,

        // used by the diagram to track which schema is selected for insertion
        selectedInsertSchemaId: null as SchemaId | null,
      }),
      getters: {
        // transforming the diagram-y data back into more generic looking data
        // TODO: ideally we just fetch it like this...
        componentsById(): Record<ComponentId, Component> {
          const diagramNodesById = _.keyBy(this.rawDiagramNodes, (n) => n.id);
          return _.mapValues(this.componentIdentificationsById, (ci) => {
            const diagramNode = diagramNodesById[ci.componentId];

            // these categories should probably have a name and a different displayName (ie "aws" vs "Amazon AWS")
            // and eventually can just assume the icon is `logo-${name}`
            const typeIcon =
              {
                AWS: "logo-aws",
                CoreOS: "logo-coreos",
                Docker: "logo-docker",
                Kubernetes: "logo-k8s",
              }[diagramNode?.category || ""] || "logo-si"; // fallback to SI logo

            return {
              id: ci.componentId,
              // TODO: return this info from the backend (and not in category)
              isGroup: diagramNode?.category === "Frames",
              displayName: diagramNode?.subtitle,
              schemaId: ci.schemaId,
              schemaName: ci.schemaName,
              schemaVariantId: ci.schemaVariantId,
              schemaVariantName: ci.schemaVariantName,
              // TODO: probably want to move this into its own store
              resource: ci.resource,
              icon: typeIcon,
              color: diagramNode?.color,
              changeStatus: this.componentChangeStatusById[ci.componentId],
            } as Component;
          });
        },
        allComponents(): Component[] {
          return _.values(this.componentsById);
        },

        selectedComponent(): Component {
          return this.componentsById[this.selectedComponentId || 0];
        },
        selectedComponentDiff(): ComponentDiff | undefined {
          return this.componentDiffsById[this.selectedComponentId || 0];
        },
        selectedComponentCode(): CodeView[] | undefined {
          return this.componentCodeViewsById[this.selectedComponentId || 0];
        },

        lastSelectedComponent(): Component {
          return this.componentsById[this.lastSelectedComponentId || 0];
        },

        diagramNodes(): DiagramNodeDef[] {
          const qualificationsStore = useQualificationsStore();
          const fixesStore = useFixesStore();
          const statusStore = useStatusStore();

          // adding logo and qualification info into the nodes
          // TODO: probably want to include logo directly
          return _.map(this.rawDiagramNodes, (node) => {
            const componentId = parseInt(node.id);

            const qualificationStatus =
              qualificationsStore.qualificationStatusByComponentId[componentId];
            const confirmationStatus =
              fixesStore.statusByComponentId[componentId];
            const changeStatus = this.componentChangeStatusById[componentId];

            const component = this.componentsById[componentId];

            const frameSocket = _.find(
              node.sockets,
              (s) => s.label === "Frame" && s.direction === "output",
            );
            const frameEdge = _.find(
              this.diagramEdges,
              (edge) =>
                edge.fromNodeId === node.id &&
                edge.fromSocketId === frameSocket?.id,
            );

            return {
              ...node,
              parentId: frameEdge ? frameEdge.toNodeId : undefined,
              isLoading:
                !!statusStore.componentStatusById[componentId]?.isUpdating,
              typeIcon: component?.icon || "logo-si",
              statusIcons: _.compact([
                changeStatusToIconMap[changeStatus],
                qualificationStatusToIconMap[qualificationStatus],
                confirmationStatusToIconMap[confirmationStatus] || {
                  icon: "minus",
                  tone: "neutral",
                },
              ]),
            };
          });
        },
        // allConnections: (state) => _.values(state.connectionsById),

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
                    const schemaVariant =
                      this.schemaVariantsById[item.schema_id];
                    const colorInt = schemaVariant?.color;
                    const color = colorInt
                      ? `#${colorInt.toString(16)}`
                      : "#777";

                    return {
                      displayName: item.name,
                      id: item.schema_id,
                      // links: item.links, // not sure this is needed?
                      color,
                    };
                  }),
                ),
              };
            }),
          );
        },

        changeStatsSummary(): Record<ComponentChangeStatus | "total", number> {
          const allChanged = _.filter(
            this.allComponents,
            (c) => !!c.changeStatus,
          );
          const grouped = _.groupBy(allChanged, (c) => c.changeStatus);
          return {
            added: grouped.added?.length || 0,
            deleted: grouped.deleted?.length || 0,
            modified: grouped.modified?.length || 0,
            total: allChanged.length,
          };
        },

        getDependentComponents: (state) => (componentId: ComponentId) => {
          // TODO: this is ugly... much of this logic is duplicated in GenericDiagram

          const connectedNodes: Record<ComponentId, ComponentId[]> = {};
          _.each(state.diagramEdges, (edge) => {
            const fromNodeId = parseInt(edge.fromNodeId);
            const toNodeId = parseInt(edge.toNodeId);
            connectedNodes[fromNodeId] ||= [];
            connectedNodes[fromNodeId].push(toNodeId);
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
          return new ApiRequest<DiagramContent>({
            url: "diagram/get_diagram",
            params: {
              ...visibilityParams,
            },
            onSuccess: (response) => {
              // for now just storing the diagram-y data
              // but I think ideally we fetch more generic component data and then transform into diagram format as necessary
              this.rawDiagramNodes = response.nodes;
              this.diagramEdges = response.edges;
            },
          });
        },
        // fetches a dropdown-style list of some component data, also including resources?
        async FETCH_COMPONENTS() {
          return new ApiRequest<{ list: LabelList<ComponentIdentification> }>({
            url: "component/list_components_identification",
            params: {
              ...visibilityParams,
            },
            onSuccess: (response) => {
              // endpoint returns dropdown-y data
              const rawIdentifications = _.map(response.list, "value");
              this.componentIdentificationsById = _.keyBy(
                rawIdentifications,
                (c) => c.componentId,
              );
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

        async FETCH_CHANGE_STATS() {
          return new ApiRequest<{ componentStats: ComponentStats }>({
            url: "change_set/get_stats",
            params: {
              ...visibilityParams,
            },
            onSuccess: (response) => {
              this.componentChangeStatusById = _.transform(
                response.componentStats.stats,
                (acc, cs) => {
                  acc[cs.componentId] = cs.componentStatus;
                },
                {} as Record<ComponentId, ComponentChangeStatus>,
              );
            },
          });
        },

        async SET_COMPONENT_DIAGRAM_POSITION(
          componentId: ComponentId,
          position: Vector2d,
        ) {
          return new ApiRequest<{ componentStats: ComponentStats }>({
            method: "post",
            url: "diagram/set_node_position",
            params: {
              nodeId: componentId,
              x: position.x.toString(),
              y: position.y.toString(),
              diagramKind: "configuration",
              ...visibilityParams,
            },
            onSuccess: (response) => {
              // record position change rather than wait for re-fetch
            },
          });
        },
        async CREATE_COMPONENT(
          schemaId: number,
          position: Vector2d,
          parentId?: number,
        ) {
          return new ApiRequest<{ node: DiagramNode }>({
            method: "post",
            url: "diagram/create_node",
            params: {
              schemaId,
              parentId,
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
          from: { componentId: ComponentId; socketId: SocketId },
          to: { componentId: ComponentId; socketId: SocketId },
        ) {
          return new ApiRequest<{ node: DiagramNode }>({
            method: "post",
            url: "diagram/create_connection",
            params: {
              fromNodeId: from.componentId,
              fromSocketId: from.socketId,
              toNodeId: to.componentId,
              toSocketId: to.socketId,
              ...visibilityParams,
            },
            onSuccess: (response) => {
              // TODO: store component details rather than waiting for re-fetch
            },
          });
        },
        async CONNECT_COMPONENT_TO_FRAME(fromNodeId: string, toNodeId: string) {
          return new ApiRequest<{ node: DiagramNode }>({
            method: "post",
            url: "diagram/connect_component_to_frame",
            params: {
              fromNodeId: parseInt(fromNodeId),
              toNodeId: parseInt(toNodeId),
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

        setSelectedComponentId(id: ComponentId | null) {
          if (!id) this.selectedComponentId = null;
          else {
            this.lastSelectedComponentId = id;
            if (this.componentsById[id]) {
              this.selectedComponentId = id;
            } else {
              // TODO: not sure... do we throw an error? Do we select the id anyway?
              this.selectedComponentId = null;
            }
          }
        },
      },
      onActivated() {
        if (!changeSetId) return;

        this.FETCH_DIAGRAM_DATA();
        this.FETCH_COMPONENTS();
        this.FETCH_AVAILABLE_SCHEMAS();
        this.FETCH_NODE_ADD_MENU();
        this.FETCH_CHANGE_STATS();

        const realtimeStore = useRealtimeStore();

        realtimeStore.subscribe(this.$id, `changeset/${changeSetId}`, [
          {
            eventType: "ChangeSetWritten",
            callback: (writtenChangeSetId) => {
              // ideally we wouldn't have to check this - since the topic subscription
              // would mean we only receive the event for this changeset already...
              // but this is fine for now
              if (writtenChangeSetId !== changeSetId) return;

              // probably want to get pushed updates instead of blindly re-fetching, but this is the first step of getting things working
              this.FETCH_DIAGRAM_DATA();
              this.FETCH_COMPONENTS();
              this.FETCH_CHANGE_STATS();
            },
          },
          {
            eventType: "CodeGenerated",
            callback: (codeGeneratedEvent) => {
              // probably ideally just push the new code over the websocket
              // but for now we'll re-fetch if the component is currently selected
              // topic subscription would also help to know if we're talking about the component in the correct changeset
              if (this.selectedComponentId === codeGeneratedEvent.componentId) {
                this.FETCH_COMPONENT_CODE(this.selectedComponentId);
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
