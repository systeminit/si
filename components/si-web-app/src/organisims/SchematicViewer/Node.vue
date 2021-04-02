<template>
  <div>
    <div
      class="absolute shadow-md cursor-move node-container node"
      :id="id"
      :class="[nodeIsSelected, nodeVisibility]"
      v-bind:style="positionStyle"
      @mousedown="selectNode()"
    >
      <span
        :ref="`${id}.socket:input`"
        :id="`${id}.socket:input`"
        class="socket-input socket node"
      />
      <span
        :ref="`${id}.socket:output`"
        :id="`${id}.socket:output`"
        class="socket-output socket node"
      />

      <div class="flex flex-col node">
        <div class="flex flex-col text-white node">
          <div class="node-title-bar node">
            <div
              class="mt-1 text-xs font-medium text-center node"
              :class="nodeTitleClasses"
            >
              {{ nodeObject.entityType }}
            </div>
          </div>

          <div class="mt-2 text-xs font-normal text-center node">
            {{ nodeObject.name }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";

import { registry } from "si-registry";
import { mapState, mapActions } from "vuex";

import { InstanceStoreContext } from "@/store";
import { PanelEventBus } from "@/atoms/PanelEventBus";
import { SiCg } from "@/api/sicg";
import { Cg2dCoordinate } from "@/api/sicg";

import { SchematicPanelStore } from "@/store/modules/schematicPanel";

import { ISchematicNode } from "@/api/sdf/model/schematic";
import { INodeObject, Node } from "@/api/sdf/model/node";
import { IEntity, Entity } from "@/api/sdf/model/entity";

import { StoresCtx } from "@/organisims/SchematicViewer.vue";

import _ from "lodash";

type NodeLayoutUpdated = boolean;

export interface NodePositionUpdateEvent {
  position: Cg2dCoordinate;
  nodeId: string;
}

interface IData {
  id: string;
  updated: number;
  nodeId: string;
  isVisible: boolean;
}

export default Vue.extend({
  name: "Node",
  props: {
    node: {
      type: Object as PropType<ISchematicNode>,
      required: true,
    },
    graphViewerId: {
      type: String,
      required: true,
    },
    storesCtx: {
      type: Object as PropType<StoresCtx>,
      required: false,
    },
    positionCtx: String,
  },
  data(): IData {
    return {
      id: this.graphViewerId + "." + this.node.node.id,
      nodeId: this.node.node.id,
      updated: 0,
      isVisible: false,
    };
  },
  mounted: function() {
    this.registerEvents();
  },
  beforeDestroy() {
    this.deRegisterEvents();
  },
  beforeUpdate: function() {
    this.updated++;
  },
  updated: function() {
    // console.log("updated")
  },
  methods: {
    async selectNode() {
      this.$emit("selectNode", this.node);
    },
    registerEvents(): void {
      PanelEventBus.$on("panel-viewport-node-update", this.updateNodePosition);
    },
    deRegisterEvents(): void {
      PanelEventBus.$off("panel-viewport-node-update", this.updateNodePosition);
    },
    redraw(event: NodeLayoutUpdated | UIEvent) {
      this.$forceUpdate();
    },
    updateNodePosition(event: NodePositionUpdateEvent) {
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
    },
    nodeIsVisible(): void {
      this.isVisible = true;
    },
  },
  computed: {
    positionStyle(): Record<string, string> {
      this.updated;
      const position = this.node?.node.positions[this.positionCtx];

      if (this.node?.node.positions[this.positionCtx]) {
        this.nodeIsVisible();
        return {
          left: `${position.x}px`,
          top: `${position.y}px`,
        };
      } else {
        return {
          left: `0px`,
          top: `0px`,
        };
      }
    },
    selectedNode(): ISchematicNode | null {
      return this.storesCtx.schematicPanelStoreCtx.state.selectedNode;
    },
    nodeIsSelected(): Record<string, boolean> {
      if (this.selectedNode && this.selectedNode.node) {
        if (this.selectedNode.node.id == this.nodeId) {
          return {
            "node-is-selected": true,
          };
        } else {
          return {
            "node-is-selected": false,
          };
        }
      }
      return {
        "node-is-selected": false,
      };
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
    nodeTitleClasses(): Record<string, boolean> {
      return {
        "input-border-gold": true,
        border: true,
        "border-t-0": true,
        "border-b-2": true,
        "border-r-0": true,
        "border-l-0": true,
      };
    },
  },
});
</script>

<style type="text/css" scoped>
/*node size and color*/
.node-container {
  width: 140px;
  height: 100px;
  background-color: #282e30;
  border-radius: 6px;
  border-width: 1px;
  border-color: transparent;
}

.node-title-bar {
  background-color: #008ed2;
  border-radius: 4px 4px 0px 0px;
}

.node-details {
  background-color: #282e30;
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

.node-is-selected {
  border-radius: 6px;
  border-width: 1px;
  border-color: #5cb1b1;
  z-index: 40;
}

.node-is-hidden {
  display: none;
}
</style>
