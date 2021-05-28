<template>
  <div>
    <div
      class="absolute shadow-md cursor-move node-container node"
      :id="id"
      :class="[
        nodeIsSelected,
        nodeVisibility,
        nodeIsDeleted,
        nodeInvalidEdgeCreating,
      ]"
      v-bind:style="positionStyle"
      @mousedown="selectNode()"
    >
      <div class="flex flex-col node">
        <div class="flex flex-col text-white node">
          <div class="node-title-bar node" :class="nodeTitleBarClasses">
            <div
              class="mt-1 text-xs font-medium text-center node"
              :class="nodeTitleClasses"
            >
              {{ nodeObject.entityType }}
            </div>
          </div>
          <div class="mt-2 mb-2 text-xs font-normal text-center node">
            {{ nodeObject.name }}
          </div>
          <div v-if="showImplementation" class="text-xs font-thin text-center">
            {{ selectedImplementationField }}
          </div>

          <div
            :ref="`${id}.socket:output`"
            :id="`${id}.socket:output`"
            :context="graphViewerId"
            :entityType="nodeObject.entityType"
            :entityId="nodeObject.id"
            :nodeId="node.node.id"
            socketName="output"
            :schematicKind="schematicKind"
            class="socket node"
            :class="outputSocketClasses"
          />

          <div class="ml-2">
            <div
              class="flex flex-row"
              v-for="input in inputs"
              :key="input.name"
            >
              <div
                :ref="`${id}.socket:${input.name}`"
                :id="`${id}.socket:${input.name}`"
                :context="graphViewerId"
                :entityType="nodeObject.entityType"
                :entityId="nodeObject.id"
                :nodeId="node.node.id"
                :socketName="input.name"
                :schematicKind="schematicKind"
                class="socket node"
                :class="inputSocketClasses"
              />
              <div class="ml-1 text-xs font-thin text-center node">
                {{ input.name }} {{ showArity(input.arity) }}
              </div>
            </div>
          </div>
        </div>
        <div class="flex flex-row ml-1 text-xs font-thin text-center node">
          <CheckCircleIcon size="1x" :class="qualificationStatusClass" />
          <OctagonIcon size="1x" :class="resourceStatusClass" />
          <PlayCircleIcon size="1x" :class="workflowStatusClass" />
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";

import { PanelEventBus } from "@/atoms/PanelEventBus";
import { Cg2dCoordinate } from "@/api/sicg";
import { ISchematicNode, SchematicKind } from "@/api/sdf/model/schematic";
import { IEntity, Entity } from "@/api/sdf/model/entity";

import _ from "lodash";
import { SiEntity, ResourceInternalHealth } from "si-entity";
import { RegistryEntry, NodeKind, registry } from "si-registry";
import {
  edgeCreating$,
  system$,
  workspace$,
  changeSet$,
  editSession$,
} from "@/observables";
import { tap, pluck, switchMap } from "rxjs/operators";
import { Arity } from "si-registry/dist/registryEntry";
import { combineLatest, of, from } from "rxjs";
import { IGetEntityRequest, AttributeDal } from "@/api/sdf/dal/attributeDal";
import {
  CheckCircleIcon,
  OctagonIcon,
  PlayCircleIcon,
} from "vue-feather-icons";

import { SchematicOrientation } from "@/organisims/SchematicViewer.vue";
import { WorkflowRunState } from "@/api/sdf/model/workflow";

type NodeLayoutUpdated = boolean;

export interface NodePositionUpdateEvent {
  position: Cg2dCoordinate;
  nodeId: string;
  positionCtx: string;
}

export enum SocketType {
  Input = "input",
  Output = "output",
}

interface NodeSocket {
  id: string;
  position: Cg2dCoordinate;
}

interface IData {
  id: string;
  nodeId: string;
  isVisible: boolean;
  invalidEdgeCreating: boolean;
  selectedImplementationField: string | null;
  sockets: NodeSocket[];
}

export default Vue.extend({
  name: "Node",
  components: {
    CheckCircleIcon,
    OctagonIcon,
    PlayCircleIcon,
  },
  props: {
    selectedNode: {
      type: Object as PropType<ISchematicNode | null>,
    },
    deploymentSelectedNode: {
      type: Object as PropType<ISchematicNode | null>,
    },
    node: {
      type: Object as PropType<ISchematicNode>,
      required: true,
    },
    schematicKind: {
      type: String as PropType<SchematicKind>,
      required: false,
      default: undefined,
    },
    graphViewerId: {
      type: String,
      required: true,
    },
    positionCtx: String,
    orientation: {
      type: String as PropType<SchematicOrientation>,
      default: "horizontal",
    },
  },
  data(): IData {
    return {
      id: this.graphViewerId + "." + this.node.node.id,
      nodeId: this.node.node.id,
      isVisible: false,
      invalidEdgeCreating: false,
      selectedImplementationField: null,
      sockets: [],
    };
  },
  subscriptions: function(this: any): Record<string, any> {
    let entity$ = this.$watchAsObservable("entity", { immediate: true }).pipe(
      pluck("newValue"),
    );
    let selectedImplementation$ = combineLatest(
      system$,
      entity$,
      workspace$,
      changeSet$,
      editSession$,
    ).pipe(
      switchMap(([system, entity, workspace, changeSet, editSession]) => {
        if (
          workspace &&
          entity &&
          system &&
          // @ts-ignore
          entity.properties[system.id] &&
          // @ts-ignore
          entity.properties[system.id]["implementation"]
        ) {
          let selectedOptionEntityId =
            // @ts-ignore
            entity.properties[system.id]["implementation"];
          let request: IGetEntityRequest = {
            workspaceId: workspace.id,
            entityId: selectedOptionEntityId as string,
          };
          if (changeSet) {
            request.changeSetId = changeSet.id;
          }
          if (editSession) {
            request.editSessionId = editSession.id;
          }
          return AttributeDal.getEntity(request);
        } else {
          return Promise.resolve({
            error: { code: 42, message: "cannot get implementation entity" },
          });
        }
      }),
      tap(reply => {
        if (reply.error) {
          if (reply.error.code == 42 || reply.error.code == 406) {
            this.selectedImplementationField = null;
          }
        } else {
          this.selectedImplementationField = `${reply.entity.entityType}: ${reply.entity.name}`;
        }
      }),
    );
    return {
      selectedImplemenation: selectedImplementation$,
      edgeCreating: edgeCreating$.pipe(
        tap(edgeCreating => {
          if (
            edgeCreating &&
            edgeCreating.schematicKind == this.schematicKind &&
            edgeCreating.entityId != this.nodeObject.id
          ) {
            let schema: RegistryEntry = this.entity.schema();
            let sourceEntitySchema = registry[edgeCreating.entityType];
            if (!sourceEntitySchema) {
              return false;
            }

            let hasValidInput = _.find(schema.inputs, input => {
              if (this.schematicKind == SchematicKind.Deployment) {
                return (
                  input.edgeKind == "deployment" &&
                  _.includes(input.types, edgeCreating.entityType)
                );
              } else if (this.schematicKind == SchematicKind.Component) {
                return (
                  input.edgeKind == "configures" &&
                  (_.includes(input.types, edgeCreating.entityType) ||
                    input.types == "dependencies" ||
                    (input.types == "implementations" &&
                      _.includes(
                        sourceEntitySchema.implements,
                        this.nodeObject.entityType,
                      )))
                );
              }
            });
            if (!hasValidInput) {
              this.invalidEdgeCreating = true;
            }
          } else {
            this.invalidEdgeCreating = false;
          }
        }),
      ),
    };
  },
  mounted: function() {
    this.registerEvents();
  },
  // updated() {
  //   this.updateSocketsPosition();
  // },
  beforeDestroy() {
    this.deRegisterEvents();
  },
  methods: {
    showArity(arity: Arity): string {
      if (Arity.One == arity) {
        return "1";
      } else if (Arity.Many == arity) {
        return "*";
      } else {
        return "";
      }
    },
    async selectNode() {
      this.$emit("selectNode", this.node);
    },
    registerEvents(): void {
      PanelEventBus.$on("panel-viewport-node-update", this.updateNodePosition);
    },
    deRegisterEvents(): void {
      PanelEventBus.$off("panel-viewport-node-update", this.updateNodePosition);
    },
    redraw(_event: NodeLayoutUpdated | UIEvent) {
      this.$forceUpdate();
    },
    updateNodePosition(event: NodePositionUpdateEvent) {
      if (event.positionCtx == this.positionCtx) {
        this.isVisible = true;
        if (event.nodeId == this.nodeId) {
          const element = document.getElementById(this.id) as HTMLElement;
          element.setAttribute(
            "style",
            "left:" +
              event.position.x +
              "px;" +
              "top:" +
              event.position.y +
              "px;",
          );
        }
      }
    },
    nodeIsVisible(): void {
      this.isVisible = true;
    },
  },
  computed: {
    qualificationStatusClass(): Record<string, any> {
      let style: Record<string, any> = {};
      if (this.node.qualifications.length > 0) {
        if (_.find(this.node.qualifications, q => q.qualified == false)) {
          style["text-red-500"] = true;
        } else {
          style["text-green-500"] = true;
        }
      } else {
        style["text-gray-600"] = true;
      }
      return style;
    },
    resourceStatusClass(): Record<string, any> {
      let style: Record<string, any> = {};
      if (
        _.find(
          Object.values(this.node.resources),
          r => r.internalHealth == ResourceInternalHealth.Error,
        )
      ) {
        style["text-red-500"] = true;
        return style;
      } else if (
        _.find(
          Object.values(this.node.resources),
          r => r.internalHealth == ResourceInternalHealth.Warning,
        )
      ) {
        style["text-yellow-500"] = true;
        return style;
      } else if (
        _.find(
          Object.values(this.node.resources),
          r => r.internalHealth == ResourceInternalHealth.Ok,
        )
      ) {
        style["text-green-500"] = true;
      } else {
        style["text-gray-600"] = true;
      }
      return style;
    },
    workflowStatusClass(): Record<string, any> {
      let style: Record<string, any> = {};
      if (
        _.find(
          Object.values(this.node.workflowRuns),
          w => w.workflowRun.state == WorkflowRunState.Failure,
        )
      ) {
        style["text-red-500"] = true;
        return style;
      } else if (
        _.find(
          Object.values(this.node.workflowRuns),
          w => w.workflowRun.state == WorkflowRunState.Running,
        )
      ) {
        style["text-blue-500"] = true;
        return style;
      } else if (
        _.find(
          Object.values(this.node.workflowRuns),
          w => w.workflowRun.state == WorkflowRunState.Success,
        )
      ) {
        style["text-green-500"] = true;
        return style;
      } else {
        style["hidden"] = true;
      }
      return style;
    },
    showImplementation(): boolean {
      return (
        !_.isNull(this.selectedImplementationField) &&
        this.schematicKind == SchematicKind.Deployment
      );
    },
    entity(): Entity {
      return SiEntity.fromJson(this.node.object as Entity);
    },
    inputs(): RegistryEntry["inputs"] {
      let inputs = _.filter(this.entity.schema().inputs, input => {
        if (this.schematicKind == SchematicKind.Deployment) {
          return input.edgeKind == "deployment";
        } else if (this.schematicKind == SchematicKind.Component) {
          return input.edgeKind == "configures";
        } else {
          return false;
        }
      });
      return inputs;
    },
    positionStyle(): Record<string, string> {
      const position = this.node?.node.positions[this.positionCtx];

      if (this.node?.node.positions[this.positionCtx]) {
        this.nodeIsVisible();
        return {
          left: `${position.x}px`,
          top: `${position.y}px`,
        };
      } else {
        this.nodeIsVisible();
        return {
          left: `0px`,
          top: `0px`,
        };
      }
    },
    inputSocketClasses(): Record<string, string> {
      let classes: Record<string, any> = {};

      switch (this.orientation) {
        case SchematicOrientation.Vertical: {
          classes["socket-input-vertical"] = true;
          classes["socket-input-horizontal"] = false;
          break;
        }

        case SchematicOrientation.Horizontal: {
          classes["socket-input-vertical"] = false;
          classes["socket-input-horizontal"] = true;
          break;
        }
      }
      return classes;
    },
    outputSocketClasses(): Record<string, string> {
      let classes: Record<string, any> = {};

      switch (this.orientation) {
        case SchematicOrientation.Vertical: {
          classes["socket-output-vertical"] = true;
          classes["socket-output-horizontal"] = false;
          break;
        }

        case SchematicOrientation.Horizontal: {
          classes["socket-output-vertical"] = false;
          classes["socket-output-horizontal"] = true;
          break;
        }
      }

      return classes;
    },
    nodeIsSelected(): Record<string, boolean> {
      if (
        this.deploymentSelectedNode &&
        this.deploymentSelectedNode.node.id == this.nodeId
      ) {
        return {
          "node-is-selected-deployment": true,
        };
      } else if (
        this.selectedNode &&
        this.selectedNode.node.id == this.nodeId
      ) {
        return {
          "node-is-selected": true,
        };
      }
      return {};
    },
    nodeInvalidEdgeCreating(): Record<string, boolean> {
      if (this.invalidEdgeCreating) {
        return { "node-disabled": true };
      } else {
        return { "node-disabled": false };
      }
    },
    nodeIsDeleted(): Record<string, boolean> {
      if (this.node.object.siStorable?.deleted) {
        return { "node-is-deleted": true };
      } else {
        return { "node-is-deleted": false };
      }
    },
    nodeVisibility(): Record<string, boolean> {
      if (!this.isVisible) {
        return { "node-is-hidden": true };
      } else {
        return { "node-is-hidden": false };
      }
    },
    nodeObject(): IEntity {
      //@ts-ignore
      return this.node.object;
    },
    nodeTitleBarClasses(): Record<string, boolean> {
      let response: Record<string, boolean> = {};
      let schema = this.entity.schema();
      if (schema.nodeKind == NodeKind.Concept) {
        response["node-concept"] = true;
      } else if (schema.nodeKind == NodeKind.Implementation) {
        response["node-implementation"] = true;
      } else if (schema.nodeKind == NodeKind.Concrete) {
        response["node-concrete"] = true;
      }
      return response;
    },
    nodeTitleClasses(): Record<string, boolean> {
      let response: Record<string, boolean> = {
        "input-border-gold": true,
        border: true,
        "border-t-0": true,
        "border-b-2": true,
        "border-r-0": true,
        "border-l-0": true,
      };

      let schema = this.entity.schema();
      if (schema.nodeKind == NodeKind.Concept) {
        response["node-concept"] = true;
      } else if (schema.nodeKind == NodeKind.Implementation) {
        response["node-implementation"] = true;
      } else if (schema.nodeKind == NodeKind.Concrete) {
        response["node-concrete"] = true;
      }
      return response;
    },
  },
});
</script>

<style type="text/css" scoped>
/*node size and color*/
.node-container {
  width: 140px;
  min-height: 100px;
  background-color: #282e30;
  border-radius: 6px;
  border-width: 1px;
  border-color: transparent;
}

.node-title-bar {
  border-radius: 4px 4px 0px 0px;
}

.node-concept {
  background-color: #008e8e;
}

.node-implementation {
  background-color: #aa11ff;
}

.node-concrete {
  background-color: #008ed2;
}

.node-details {
  background-color: #282e30;
}

.node-disabled {
  filter: brightness(50%) saturate(50%);
}

.socket-input {
  display: block;
  height: 12px;
  width: 12px;
  background-color: #282e30;
  border-radius: 50%;
  border-width: 1px;
  border-color: #008ed2;
  position: absolute;
  top: 0px;
  left: 62px;
  margin-top: -6px;
}

.socket-input-vertical {
  display: block;
  height: 12px;
  width: 12px;
  background-color: #282e30;
  border-radius: 50%;
  border-width: 1px;
  border-color: #008ed2;
  position: absolute;
  top: 0px;
  left: 62px;
  margin-top: -6px;
}

.socket-input-horizontal {
  display: block;
  height: 12px;
  width: 12px;
  background-color: #282e30;
  border-radius: 50%;
  border-width: 1px;
  border-color: #008ed2;
  position: absolute;
  left: -6px;
}

.socket-output {
  display: block;
  height: 12px;
  width: 12px;
  background-color: #282e30;
  border-radius: 50%;
  border-width: 1px;
  border-color: #008ed2;
  position: absolute;
  bottom: 0px;
  left: 62px;
  margin-bottom: -6px;
}

.socket-output-vertical {
  display: block;
  height: 12px;
  width: 12px;
  background-color: #282e30;
  border-radius: 50%;
  border-width: 1px;
  border-color: #008ed2;
  position: absolute;
  bottom: 0px;
  left: 62px;
  margin-bottom: -6px;
}

.socket-output-horizontal {
  display: block;
  height: 12px;
  width: 12px;
  background-color: #282e30;
  border-radius: 50%;
  border-width: 1px;
  border-color: #008ed2;
  position: absolute;
  top: 30px;
  left: 132px;
  margin-bottom: -6px;
}

.node-is-selected {
  border-radius: 6px;
  border-width: 1px;
  border-color: #5cb1b1;
  z-index: 40;
}

.node-is-selected-deployment {
  border-radius: 6px;
  border-width: 1px;
  border-color: #00ffff;
  z-index: 40;
}

.node-is-deleted {
  border-radius: 6px;
  border-width: 1px;
  border-color: #e21010;
}

.node-is-hidden {
  display: none;
}
</style>
