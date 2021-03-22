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

      <div class="flex flex-col select-none node">
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
/**
 * NodeObject
 *
 * A Node
 *  id
 *  kind # the lkind of node, defines the color of the node actions, and more...
 *  name # the name displayed on a node.
 *  inputs [] # a list of input sockets setting properties on this node.
 *  outputs [] # a list of output sockets exposing properties from this this node.
 *  connections [] # a list of connections between this and other node(s).
 *  position # the node position (x,y). Position has to be relative to the main coordinate system.
 *  data # the actual node data.
 *  Note: the inputs and outputs list could be combined into a single list: sockets[]. A socket would be of kind input or output.
 *
 * A Socket (input or output)
 *  id
 *  name # maps to a property on this node.
 *  kind # the kind of socket, input or output.
 *  type # the type of socket, string, int, float, bool, service, k8s-blah, ...
 *  position # to draw connections.
 *
 * A Connection (input or output) - local representation of a connection.
 *  id
 *  socket # a socket on this node.
 *  path # a socket to connect with.
 * kind i/o
 *
 * A Connection (input or output) - global representation of a connection, what goes in the connectionList
 *  id
 *  source # a node socket.
 *  destination # a socket to connect with.
 *
 */

import Vue, { PropType } from "vue";
import { registry } from "si-registry";
import { mapState, mapActions } from "vuex";
import _ from "lodash";

// import { RootStore } from "@/store";
import { ISchematicNode } from "@/api/sdf/model/schematic";
import { PanelEventBus } from "@/atoms/PanelEventBus";

import { SchematicPanelStore } from "@/store/modules/schematicPanel";

import { InstanceStoreContext } from "@/store";

import { INodeObject, Node } from "@/api/sdf/model/node";
import { IEntity, Entity } from "@/api/sdf/model/entity";
import { StoresCtx } from "@/organisims/GraphViewer.vue";

import { SiCg } from "@/api/sicg";
import { Cg2dCoordinate } from "@/api/sicg";

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
      if (event.nodeId == this.nodeId) {
        let element = document.getElementById(this.id) as HTMLElement;
        SiCg.cgSetElementPosition(element, event.position);
      }
    },
    nodeIsVisible(): void {
      this.isVisible = true;
    },
  },
  computed: {
    positionStyle(): Record<string, string> {
      this.updated;

      let context = "AAA";
      let position = this.node?.node.positions[context];

      if (this.node?.node.positions[context]) {
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
