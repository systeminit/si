<template>
  <div>
    <div
      class="absolute flex shadow-md cursor-move node-container node"
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
      <div class="flex flex-col justify-between flex-grow">
        <div class="flex flex-col node">
          <div
            class="items-center w-full node-title-bar node"
            :class="nodeTitleBarClasses"
          >
            <div
              class="flex flex-row items-center justify-between node title-bar-content"
            >
              <div class="w-1" />

              <div
                class="flex text-xs font-medium text-center text-white align-middle node"
              >
                {{ nodeObject.entityType }}
              </div>

              <div
                class="flex items-center justify-center rounded-full diff-count-content node"
                v-if="diffCount == 0"
              >
                <div
                  class="font-medium text-center align-middle diff-count-text node"
                >
                  {{ diffCount }}
                </div>
              </div>
              <div class="w-1" v-else />
            </div>

            <div class="w-full">
              <div class="status-bar" :class="statusBarClass" />
            </div>
          </div>

          <div class="flex flex-row justify-center mx-1 mt-2 mb-2 node">
            <div class="text-center node-name node">
              {{ nodeObject.name }}
            </div>
            <!-- <div class="w-3 node">
              <PlayCircleIcon
                size="1x"
                :class="workflowStatusClass"
                class="text-sm font-thin text-center node"
              />
            </div> -->
          </div>

          <div
            v-if="showImplementation"
            class="text-xs font-thin text-center node"
          >
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

          <div class="ml-2 -mt-1">
            <div
              class="flex flex-row mt-1"
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
                :class="[inputSocketClasses, socketTypeClass(input.name)]"
              />
              <div class="text-center socket-name node">
                {{ input.name }} {{ showArity(input.arity) }}
              </div>
            </div>
          </div>
        </div>

        <div
          class="flex flex-row justify-end mx-1 mb-1 text-sm font-thin text-center node"
        >
          <CheckSquareIcon
            size="1x"
            class="mr-1 node"
            :class="qualificationStatusClass"
          />

          <BoxIcon size="1x" class="node" :class="resourceStatusClass" />
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
  CheckSquareIcon,
  BoxIcon,
  // PlayCircleIcon
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
    CheckSquareIcon,
    BoxIcon,
    // PlayCircleIcon,
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
      editSession: editSession$,
      changeSet: changeSet$,
      workspace: workspace$,
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
    socketTypeClass(name: String): Record<string, any> {
      let style: Record<string, any> = {};
      if (
        name == "k8sDeployment" ||
        name == "k8sNamespace" ||
        name == "k8sService"
      ) {
        style["socket-component-k8s"] = true;
      } else if (
        name == "awsAccessKey" ||
        name == "awsEks" ||
        name == "awsEksCluster" ||
        name == "awsRegion"
      ) {
        style["socket-component-aws"] = true;
      } else if (
        name == "azureAks" ||
        name == "azureResourceGroup" ||
        name == "azureServicePrincipal" ||
        name == "azureLocation" ||
        name == "azureAksCluster"
      ) {
        style["socket-component-az"] = true;
      } else if (
        name == "aws" ||
        name == "kubernetesService" ||
        name == "azure"
      ) {
        style["socket-implementation"] = true;
      } else if (name == "dockerImage") {
        style["socket-component-docker"] = true;
      } else if (name == "implementations") {
        style["socket-implementations"] = true;
      } else if (name == "service") {
        style["socket-service"] = true;
      } else if (name == "kubernetesCluster") {
        style["socket-kubernetes-cluster"] = true;
      }
      return style;
    },
    showArity(arity: Arity): string {
      if (Arity.One == arity) {
        return "";
      } else if (Arity.Many == arity) {
        return "";
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
    diffCount(): Number | null {
      //@ts-ignore
      if (this.selectedImplemenation && this.selectedImplemenation.diff) {
        //@ts-ignore
        return this.selectedImplemenation.diff.length;
      }
      return null;
    },
    qualificationStatusClass(): Record<string, any> {
      let style: Record<string, any> = {};
      if (this.node.qualifications.length > 0) {
        if (_.find(this.node.qualifications, q => q.qualified == false)) {
          style["error"] = true;
        } else {
          style["ok"] = true;
        }
      } else {
        style["unknown"] = true;
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
        style["error"] = true;
        return style;
      } else if (
        _.find(
          Object.values(this.node.resources),
          r => r.internalHealth == ResourceInternalHealth.Warning,
        )
      ) {
        style["warning"] = true;
        return style;
      } else if (
        _.find(
          Object.values(this.node.resources),
          r => r.internalHealth == ResourceInternalHealth.Ok,
        )
      ) {
        style["ok"] = true;
      } else {
        style["unknown"] = true;
      }
      return style;
    },
    statusBarClass(): Record<string, any> {
      let style: Record<string, any> = {};
      if (
        _.find(
          Object.values(this.node.workflowRuns),
          w => w.workflowRun.state == WorkflowRunState.Failure,
        )
      ) {
        style["error-fill"] = true;
        return style;
      } else if (
        _.find(
          Object.values(this.node.workflowRuns),
          w => w.workflowRun.state == WorkflowRunState.Running,
        )
      ) {
        style["running-fill"] = true;
        style["running-animation"] = true;
        return style;
      } else if (
        _.find(
          Object.values(this.node.workflowRuns),
          w => w.workflowRun.state == WorkflowRunState.Success,
        )
      ) {
        style["ok-fill"] = true;
        return style;
      } else {
        style["hidden"] = true;
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
        style["error"] = true;
        return style;
      } else if (
        _.find(
          Object.values(this.node.workflowRuns),
          w => w.workflowRun.state == WorkflowRunState.Running,
        )
      ) {
        style["running"] = true;
        return style;
      } else if (
        _.find(
          Object.values(this.node.workflowRuns),
          w => w.workflowRun.state == WorkflowRunState.Success,
        )
      ) {
        style["ok"] = true;
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

      let schema = this.entity.schema();

      switch (this.orientation) {
        case SchematicOrientation.Vertical: {
          classes["socket-output-vertical"] = true;
          classes["socket-output-horizontal"] = false;

          if (schema.entityType == "service") {
            classes["socket-service"] = true;
          } else if (schema.entityType == "kubernetesCluster") {
            classes["socket-kubernetes-cluster"] = true;
          } else if (schema.entityType == "cloudProvider") {
            classes["socket-cloud-provider"] = true;
          }

          break;
        }

        case SchematicOrientation.Horizontal: {
          classes["socket-output-vertical"] = false;
          classes["socket-output-horizontal"] = true;

          if (schema.entityType == "service") {
            classes["socket-service"] = true;
          } else if (schema.entityType == "kubernetesCluster") {
            classes["socket-kubernetes-cluster"] = true;
          } else if (schema.entityType == "cloudProvider") {
            classes["socket-cloud-provider"] = true;
          } else if (
            schema.entityType == "k8sDeployment" ||
            schema.entityType == "k8sNamespace" ||
            schema.entityType == "k8sService"
          ) {
            classes["socket-component-k8s"] = true;
          } else if (
            schema.entityType == "awsAccessKey" ||
            schema.entityType == "awsEksCluster" ||
            schema.entityType == "awsRegion"
          ) {
            classes["socket-component-aws"] = true;
          } else if (
            schema.entityType == "azureResourceGroup" ||
            schema.entityType == "azureServicePrincipal" ||
            schema.entityType == "azureLocation" ||
            schema.entityType == "azureAksCluster"
          ) {
            classes["socket-component-az"] = true;
          } else if (
            schema.entityType == "aws" ||
            schema.entityType == "kubernetesService" ||
            schema.entityType == "azure" ||
            schema.entityType == "azureAks" ||
            schema.entityType == "awsEks"
          ) {
            classes["socket-implementations"] = true;
          } else if (schema.entityType == "dockerImage") {
            classes["socket-component-docker"] = true;
          }

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
      if (schema.entityType == "service") {
        response["node-service"] = true;
      } else if (schema.entityType == "kubernetesCluster") {
        response["node-kubernetes-cluster"] = true;
      } else if (schema.entityType == "cloudProvider") {
        response["node-cloud-provider"] = true;
      } else if (
        schema.entityType == "k8sDeployment" ||
        schema.entityType == "k8sNamespace" ||
        schema.entityType == "k8sService"
      ) {
        response["node-component-k8s"] = true;
      } else if (
        schema.entityType == "awsAccessKey" ||
        schema.entityType == "awsEksCluster" ||
        schema.entityType == "awsRegion"
      ) {
        response["node-component-aws"] = true;
      } else if (
        schema.entityType == "azureResourceGroup" ||
        schema.entityType == "azureServicePrincipal" ||
        schema.entityType == "azureLocation" ||
        schema.entityType == "azureAksCluster"
      ) {
        response["node-component-az"] = true;
      } else if (
        schema.entityType == "aws" ||
        schema.entityType == "kubernetesService" ||
        schema.entityType == "azure" ||
        schema.entityType == "azureAks" ||
        schema.entityType == "awsEks"
      ) {
        response["node-implementation"] = true;
      } else if (schema.entityType == "dockerImage") {
        response["node-component-docker"] = true;
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

.arity-one {
  /* color: #DBDBDB; */
  /* border-color: #DBDBDB; */
  background-color: #282e30;
  border-color: #008ed2;
  /* border: none; */
}

.arity-many {
  /* color: #EBEDBE; */
  /* border-color: #EBEDBE; */
  background-color: #004c70;
  border-color: #008ed2;
  /* border: none; */
}
.node-name {
  font-size: 10px;
  color: #ededed;
  font-weight: 500;
}

.socket-name {
  margin-left: 1px;
  font-size: 9px;
  /* color: #DBDBDB; */
}

.status-bar {
  height: 3px;
}

.qualification-content {
  height: 14px;
  width: 14px;
  margin-left: 2px;
  background-color: #282e30;
  border-radius: 2px 2px 2px 2px;
}

.qualification-icon {
  padding-right: 1px;
}

.title-bar-empty-space {
  height: 14px;
  width: 14px;
}

.diff-count-content {
  height: 14px;
  width: 14px;
  background-color: #282e30;
  margin-right: 2px;
}

.diff-count-text {
  color: #ce7f3e;
  padding-right: 0.5px;
  font-size: 0.5em;
}

.node-container {
  width: 140px;
  min-height: 100px;
  background-color: #282e30;
  border-radius: 4px 4px 4px 4px;
  border-width: 1px;
  border-color: transparent;
}

.node-title-bar {
  border-radius: 4px 4px 0px 0px;
}

.title-bar-content {
  padding-top: 4px;
  padding-bottom: 2px;
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

.node-service {
  /* background-color: #00C5D2; */
  background-color: #00b0bc;
}

.socket-service {
  background-color: #00b0bc;
  border-color: #00d6e4;
}

.node-kubernetes-cluster {
  background-color: #80c037;
}

.socket-kubernetes-cluster {
  background-color: #80c037;
  border-color: #99e642;
}

.node-cloud-provider {
  background-color: #ed6800;
}

.socket-cloud-provider {
  background-color: #ed6800;
  border-color: #ff7000;
}

.node-component-k8s {
  background-color: #921ed6;
}

.socket-component-k8s {
  background-color: #921ed6;
  border-color: #c664ff;
}

.node-component-aws {
  /* background-color: #D6C51E; */
  background-color: #c0b011;
}

.socket-component-aws {
  background-color: #c0b011;
  border-color: #ffea17;
}

.node-component-az {
  /* background-color: #1ED6A3; */
  background-color: #18b08d;
}

.socket-component-az {
  background-color: #18b08d;
  border-color: #20eebf;
}

.node-component-docker {
  background-color: #1e88d6;
}

.socket-component-docker {
  background-color: #1e88d6;
  border-color: #7ac7ff;
}

.node-implementation {
  background-color: #d61e8c;
}

.socket-implementations {
  background-color: #d61e8c;
  border-color: #f873c2;
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
  /* background-color: #282e30; */
  border-radius: 50%;
  border-width: 1px;
  /* border-color: #008ed2; */
  position: absolute;
  top: 0px;
  left: 62px;
  margin-top: -6px;
}

.socket-input-vertical {
  display: block;
  height: 12px;
  width: 12px;
  /* background-color: #282e30; */
  /* background-color: red; */
  border-radius: 50%;
  border-width: 1px;
  /* border-color: #008ed2; */
  position: absolute;
  top: 0px;
  left: 62px;
  margin-top: -6px;
}

.socket-input-horizontal {
  display: block;
  height: 12px;
  width: 12px;
  border-radius: 50%;
  border-width: 1px;
  position: absolute;
  left: -6px;
}

.socket-output {
  display: block;
  height: 12px;
  width: 12px;
  /* background-color: #282e30; */
  /* background-color: #004C70; */
  border-radius: 50%;
  border-width: 1px;
  /* border-color: #008ed2; */

  position: absolute;
  bottom: 0px;
  left: 62px;
  margin-bottom: -6px;
}

.socket-output-vertical {
  display: block;
  height: 12px;
  width: 12px;
  /* background-color: #282e30; */
  /* background-color: #004C70; */
  border-radius: 50%;
  border-width: 1px;
  /* border-color: #008ed2; */
  position: absolute;
  bottom: 0px;
  left: 62px;
  margin-bottom: -6px;
}

.socket-output-horizontal {
  display: block;
  height: 12px;
  width: 12px;
  /* background-color: #282e30; */
  /* background-color: #004C70; */
  border-radius: 50%;
  border-width: 1px;
  /* border-color: #008ed2; */
  position: absolute;
  top: 54px;
  left: 132px;
  margin-bottom: -6px;
}

.node-is-selected {
  border-radius: 6px;
  border-width: 1px;
  border-color: #b7d2d4;
  z-index: 40;
}

.node-is-selected-deployment {
  border-radius: 6px;
  border-width: 1px;
  border-color: #b7d2d4;
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

.ok {
  color: #86f0ad;
}

.warning {
  color: #f0d286;
}

.error {
  color: #f08686;
}

.unknown {
  color: #bbbbbb;
}

.running {
  color: #69bef0;
}

.ok-fill {
  background-color: #86f0ad;
}

.warning-fill {
  background-color: #f0d286;
}

.error-fill {
  background-color: #f08686;
}

.unknown-fill {
  background-color: #bbbbbb;
}

.running-fill {
  background-color: #69bef0;
}

.running-animation {
  animation: pulse 0.5s cubic-bezier(0, 0.19, 0.25, 1.01) infinite;
  @keyframes pulse {
    0% {
      opacity: 1;
    }
    100% {
      opacity: 0;
    }
  }
}
</style>
