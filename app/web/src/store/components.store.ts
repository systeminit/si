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
import { ComponentStats, ComponentStatus } from "@/api/sdf/dal/change_set";
import { LabelList } from "@/api/sdf/dal/label_list";
import { ComponentIdentification } from "@/api/sdf/dal/component";
import { Resource } from "@/api/sdf/dal/resource";
import { useChangeSetsStore } from "./change_sets.store";
import { useRealtimeStore } from "./realtime/realtime.store";
import {
  QualificationStatus,
  useQualificationsStore,
} from "./qualifications.store";

export type ComponentId = number;
type Component = {
  id: ComponentId;
  displayName: string;
  schemaName: string;
  schemaId: number;
  schemaVariantId: number;
  schemaVariantName: string;
  color: string;
  changeStatus?: ComponentStatus;
  // TODO: probably want to move this to a different store and not load it all the time
  resource: Resource;
};

type SocketId = number;

type SchemaId = number;

type NodeAddMenu = {
  displayName: string;
  schemas: {
    id: SchemaId;
    displayName: string;
    color: string;
  }[];
}[];

const qualificationStatusToIconMap: Record<
  QualificationStatus,
  DiagramStatusIcon
> = {
  success: { icon: "check", tone: "success" },
  failure: { icon: "alert", tone: "error" },
  running: { icon: "loading", tone: "info" },
};

export const useComponentsStore = () => {
  const changeSetsStore = useChangeSetsStore();
  const changeSetId = changeSetsStore.selectedChangeSetId;
  const workspaceId = changeSetsStore.selectedWorkspaceId;
  return addStoreHooks(
    defineStore(`cs${changeSetId}/components`, {
      state: () => ({
        // components within this changeset
        // componentsById: {} as Record<ComponentId, Component>,
        // connectionsById: {} as Record<ConnectionId, Connection>,
        // added / deleted / modified
        componentIdentificationsById: {} as Record<
          ComponentId,
          ComponentIdentification
        >,
        componentChangeStatusById: {} as Record<ComponentId, ComponentStatus>,

        rawDiagramNodes: [] as DiagramNodeDef[],
        diagramEdges: [] as DiagramEdgeDef[],
        schemaVariantsById: {} as Record<SchemaId, DiagramSchemaVariant>,
        rawNodeAddMenu: [] as MenuItem[],

        selectedComponentId: null as ComponentId | null,
      }),
      getters: {
        // transforming the diagram-y data back into more generic looking data
        // TODO: ideally we just fetch it like this...
        componentsById(): Record<ComponentId, Component> {
          const diagramNodesById = _.keyBy(this.rawDiagramNodes, (n) => n.id);
          return _.mapValues(this.componentIdentificationsById, (ci) => {
            const diagramNode = diagramNodesById[ci.componentId];
            return {
              id: ci.componentId,
              displayName: diagramNode?.subtitle,
              schemaId: ci.schemaId,
              schemaName: ci.schemaName,
              schemaVariantId: ci.schemaVariantId,
              schemaVariantName: ci.schemaVariantName,
              resource: ci.resource,
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

        diagramNodes(): DiagramNodeDef[] {
          // adding logo and qualification info into the nodes
          // TODO: probably want to include logo directly
          return _.map(this.rawDiagramNodes, (node) => {
            // Default to "si" if we do not have a logo.
            let typeIcon = "si";
            if (
              node.category === "AWS" ||
              node.category === "CoreOS" ||
              node.category === "Docker" ||
              node.category === "Kubernetes"
            ) {
              typeIcon = node.category;
            }

            const qualificationsStore = useQualificationsStore();
            const qualificationStatus =
              qualificationsStore.qualificationStatusByComponentId[
                parseInt(node.id)
              ];

            return {
              ...node,
              typeIcon,
              statusIcons: _.compact([
                qualificationStatusToIconMap[qualificationStatus],
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
                    // ignoring "link" items - dont think these are relevant at the moment
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

        changeStatsSummary(): Record<ComponentStatus | "total", number> {
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

        // other store getters - just more convenient to keep in getters for re-use throughout /////////////
        selectedSystemId() {
          return undefined;
        },
      },
      actions: {
        // TODO: change these endpoints to return a more complete picture of component data in one call

        // actually fetches diagram-style data, but we have a computed getter to turn back into more generic component data above
        async FETCH_DIAGRAM_DATA() {
          return new ApiRequest<DiagramContent>({
            method: "get",
            url: "diagram/get_diagram",
            params: {
              visibility_change_set_pk: changeSetId,
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
            method: "get",
            url: "component/list_components_identification",
            params: {
              visibility_change_set_pk: changeSetId,
              workspaceId,
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
            method: "get",
            // TODO: probably switch to something like GET `/workspaces/:id/schemas`?
            url: "diagram/list_schema_variants",
            params: {
              workspaceId,
              visibility_change_set_pk: changeSetId,
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
              workspaceId,
              visibility_change_set_pk: changeSetId,
            },
            onSuccess: (response) => {
              this.rawNodeAddMenu = response;
            },
          });
        },

        async FETCH_CHANGE_STATS() {
          return new ApiRequest<{ componentStats: ComponentStats }>({
            method: "get",
            url: "change_set/get_stats",
            params: {
              visibility_change_set_pk: changeSetId,
            },
            onSuccess: (response) => {
              this.componentChangeStatusById = _.transform(
                response.componentStats.stats,
                (acc, cs) => {
                  acc[cs.componentId] = cs.componentStatus;
                },
                {} as Record<ComponentId, ComponentStatus>,
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
              systemId: this.selectedSystemId,
              visibility_change_set_pk: changeSetId,
            },
            onSuccess: (response) => {
              // record position change rather than wait for re-fetch
            },
          });
        },
        async CREATE_COMPONENT(schemaId: number, position: Vector2d) {
          return new ApiRequest<{ node: DiagramNode }>({
            method: "post",
            url: "diagram/create_node",
            params: {
              schemaId,
              x: position.x.toString(),
              y: position.y.toString(),
              visibility_change_set_pk: changeSetId,
              workspaceId,
            },
            onSuccess: (response) => {
              // TODO: store componenet details rather than waiting for re-fetch
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

              visibility_change_set_pk: changeSetId,
              workspaceId,
            },
            onSuccess: (response) => {
              // TODO: store componenet details rather than waiting for re-fetch
            },
          });
        },

        setSelectedComponentId(id: ComponentId | null) {
          if (!id) this.selectedComponentId = null;
          else {
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
              // ideally we wouldnt have to check this - since the topic subscription
              // would mean we only receive the event for this changeset already...
              // but this is fine for now
              if (writtenChangeSetId !== changeSetId) return;

              // probably want to get pushed updates instead of blindly refetching, but this is the first step of getting things working
              this.FETCH_DIAGRAM_DATA();
              this.FETCH_COMPONENTS();
              this.FETCH_CHANGE_STATS();
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
