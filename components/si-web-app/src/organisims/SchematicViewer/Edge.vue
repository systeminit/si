<template>
  <div :id="edge.id" v-if="sourceNode && destinationNode">
    <svg height="100%" width="100%" class="absolute" :id="id">
      <line
        v-if="sourceSocketPosition && destinationSocketPosition"
        :id="lineId"
        :x1="sourceSocketPosition.x"
        :y1="sourceSocketPosition.y"
        :x2="destinationSocketPosition.x"
        :y2="destinationSocketPosition.y"
        style="stroke:rgb(71,99,113); stroke-width:2"
      />
    </svg>
  </div>
</template>

<script lang="ts">
/**
 * - Connections between nodes.
 *  We should have a connectionList component that is a single SVG with a line per connection.
 *  A connection should take a sourceNodeSocket and destinationNodeSocket
 *  We should extract the head and tail position from the sockets
 *
 *  This proto is simulating some of that and will need to be refactored.
 *
 */

import Vue, { PropType } from "vue";

import { InstanceStoreContext } from "@/store";
import { PanelEventBus } from "@/atoms/PanelEventBus";
import { Cg2dCoordinate } from "@/api/sicg";

import { SchematicPanelStore } from "@/store/modules/schematicPanel";

import { Edge } from "@/api/sdf/model/edge";
import { ISchematicNode, Schematic } from "@/api/sdf/model/schematic";

const INPUT_SOCKET_OFFSET = {
  x: 68.5, // node center.
  y: 0, // top line of a node
};

const OUTPUT_SOCKET_OFFSET = {
  x: 68.5, // node center.
  y: 100, // bottom line of a node
};

export interface EdgeTemporary {
  id: string;
  headVertex: {
    nodeId: string;
  };
  tailVertex: {
    nodeId: string;
  };
}

interface EdgePositionUpdate {
  sourceNodeId: string;
  destinationNodeId: string;
  nodeId?: never;
  nodePosition?: never;
}

interface NodePositionUpdate {
  sourceNodeId?: never;
  destinationNodeId?: never;
  nodeId: string;
  nodePosition: Cg2dCoordinate;
}

export type EdgePostionUpdateEvent = EdgePositionUpdate | NodePositionUpdate;

interface IData {
  id: string;
  lineId: string;
  updated: number;
  isVisible: boolean;
}

export default Vue.extend({
  name: "Edge",
  props: {
    edge: {
      type: (Object as PropType<Edge> | Object) as PropType<EdgeTemporary>,
      required: false,
    },
    schematicPanelStoreCtx: {
      type: Object as PropType<InstanceStoreContext<SchematicPanelStore>>,
      required: true,
    },
    graphViewerId: {
      type: String,
      required: true,
    },
    positionCtx: String,
  },
  data(): IData {
    return {
      id: this.graphViewerId + "." + this.edge.id,
      lineId: this.graphViewerId + "." + this.edge.id + "." + "svg:line",
      updated: 0,
      isVisible: false,
    };
  },
  computed: {
    sourceNode(): ISchematicNode | null {
      if (
        this.schematicPanelStoreCtx &&
        this.schematicPanelStoreCtx.state &&
        this.schematicPanelStoreCtx.state.schematic &&
        this.schematicPanelStoreCtx.state.schematic != undefined
      ) {
        return this.schematicPanelStoreCtx.state.schematic.nodes[
          this.edge.tailVertex.nodeId
        ] as ISchematicNode;
      } else {
        return null;
      }
    },
    destinationNode(): ISchematicNode | null {
      if (
        this.schematicPanelStoreCtx &&
        this.schematicPanelStoreCtx.state &&
        this.schematicPanelStoreCtx.state.schematic &&
        this.schematicPanelStoreCtx.state.schematic != undefined
      ) {
        return this.schematicPanelStoreCtx.state.schematic?.nodes[
          this.edge.headVertex.nodeId
        ] as ISchematicNode;
      } else {
        return null;
      }
    },
    sourceSocketPosition(): Cg2dCoordinate | undefined {
      const context = this.positionCtx;
      if (this.sourceNode) {
        const sourceNodePosition: Cg2dCoordinate = {
          x: Number(this.sourceNode.node.positions[context].x),
          y: Number(this.sourceNode.node.positions[context].y),
        };
        return {
          x: sourceNodePosition.x + OUTPUT_SOCKET_OFFSET.x,
          y: sourceNodePosition.y + OUTPUT_SOCKET_OFFSET.y,
        };
      } else {
        return undefined;
      }
    },

    destinationSocketPosition(): Cg2dCoordinate | undefined {
      const context = this.positionCtx;
      if (
        this.destinationNode &&
        this.destinationNode.node.positions[context]
      ) {
        const destinationNodePosition: Cg2dCoordinate = {
          x: Number(this.destinationNode.node.positions[context].x),
          y: Number(this.destinationNode.node.positions[context].y),
        };
        return {
          x: destinationNodePosition.x + INPUT_SOCKET_OFFSET.x,
          y: destinationNodePosition.y + INPUT_SOCKET_OFFSET.y,
        };
      } else {
        return undefined;
      }
    },
  },
  mounted() {
    this.registerEvents();
  },
  beforeDestroy() {
    this.deRegisterEvents();
  },
  beforeUpdate: function() {
    this.updated++;
  },
  methods: {
    registerEvents(): void {
      PanelEventBus.$on(
        "panel-viewport-edge-update",
        this.updateSvgLinePosition,
      );
      PanelEventBus.$on(
        "panel-viewport-edge-update" + "." + this.id,
        this.updateSvgLinePosition,
      );
      PanelEventBus.$on(
        this.eventIdForNodeId(this.sourceNode?.node.id as string),
        this.updateSvgLinePosition,
      );
      PanelEventBus.$on(
        this.eventIdForNodeId(this.destinationNode?.node.id as string),
        this.updateSvgLinePosition,
      );
    },
    deRegisterEvents(): void {
      PanelEventBus.$off(
        "panel-viewport-edge-update",
        this.updateSvgLinePosition,
      );
      PanelEventBus.$off(
        "panel-viewport-edge-update" + "." + this.id,
        this.updateSvgLinePosition,
      );
      PanelEventBus.$off(
        this.eventIdForNodeId(this.sourceNode?.node.id as string),
        this.updateSvgLinePosition,
      );
      PanelEventBus.$off(
        this.eventIdForNodeId(this.destinationNode?.node.id as string),
        this.updateSvgLinePosition,
      );
    },
    eventIdForNodeId(nodeId: string): string {
      return (
        "panel-viewport-edge-update" + "." + this.graphViewerId + "." + nodeId
      );
    },
    updateSvgLinePosition(event: EdgePostionUpdateEvent) {
      if (this.sourceNode && this.destinationNode) {
        const element = document.getElementById(this.lineId) as HTMLElement;

        if (event.nodeId == this.sourceNode.node.id) {
          element.setAttribute(
            "x1",
            String(event.nodePosition.x + OUTPUT_SOCKET_OFFSET.x),
          );
          element.setAttribute(
            "y1",
            String(event.nodePosition.y + OUTPUT_SOCKET_OFFSET.y),
          );
        } else if (event.nodeId == this.destinationNode.node.id) {
          element.setAttribute(
            "x2",
            String(event.nodePosition.x + INPUT_SOCKET_OFFSET.x),
          );
          element.setAttribute(
            "y2",
            String(event.nodePosition.y + INPUT_SOCKET_OFFSET.y),
          );
        }
      }
    },
    getNode(nodeId: string): ISchematicNode | undefined {
      if (this.schematicPanelStoreCtx.state.schematic?.nodes[nodeId]) {
        return this.schematicPanelStoreCtx.state.schematic?.nodes[nodeId];
      }
    },
  },
});
</script>
