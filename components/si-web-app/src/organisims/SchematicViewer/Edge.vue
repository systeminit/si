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

import _ from "lodash";

import { RegistryEntry } from "si-registry";
import { SiEntity } from "si-entity";

import { Cg2dCoordinate } from "@/api/sicg";
import { Edge } from "@/api/sdf/model/edge";
import { Entity } from "@/api/sdf/model/entity";
import { ISchematicNode, Schematic } from "@/api/sdf/model/schematic";

import { PanelEventBus } from "@/atoms/PanelEventBus";

import { SchematicOrientation } from "@/organisims/SchematicViewer.vue";

const INPUT_SOCKET_OFFSET_VERTICAL = {
  x: 68.5,
  y: 0,
};

const INPUT_SOCKET_OFFSET_HORIZONTAL = {
  x: 0,
  y: 61,
};

const OUTPUT_SOCKET_OFFSET_VERTICAL = {
  x: 68.5,
  y: 100,
};

const OUTPUT_SOCKET_OFFSET_HORIZONTAL = {
  x: 138,
  y: 36,
};

const INPUT_SOCKET_ALIGNMENT_OFSET = {
  x: 0,
  y: 16,
};

// ----------------------------------------
// Connection
export interface Connection {
  kind: ConnectionKind;
  from: Source;
  to: Source;
  edge: Edge;
}

export interface Source {
  nodeId: string;
  socketId: string;
}

export enum ConnectionKind {
  Deployment = "deployment",
  Implementation = "implementation",
}
// ----------------------------------------

export interface EdgeTemporary {
  id: string;
  headVertex: {
    nodeId: string;
    socket?: string;
  };
  tailVertex: {
    nodeId: string;
    socket?: string;
  };
}

interface EdgePositionUpdate {
  sourceNodeId: string;
  destinationNodeId: string;
  nodeId?: never;
  nodePosition?: never;
  positionCtx: string;
}

interface NodePositionUpdate {
  sourceNodeId?: never;
  destinationNodeId?: never;
  nodeId: string;
  nodePosition: Cg2dCoordinate;
  positionCtx: string;
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
    schematic: {
      type: Object as PropType<Schematic>,
      required: false,
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
      id: this.graphViewerId + "." + this.edge.id,
      lineId: this.graphViewerId + "." + this.edge.id + "." + "svg:line",
      updated: 0,
      isVisible: false,
    };
  },
  computed: {
    sourceNode(): ISchematicNode | null {
      if (this.schematic) {
        return this.schematic.nodes[
          this.edge.tailVertex.nodeId
        ] as ISchematicNode;
      } else {
        return null;
      }
    },
    destinationNode(): ISchematicNode | null {
      if (this.schematic) {
        return this.schematic.nodes[
          this.edge.headVertex.nodeId
        ] as ISchematicNode;
      } else {
        return null;
      }
    },
    sourceSocketPosition(): Cg2dCoordinate | undefined {
      const context = this.positionCtx;
      if (this.sourceNode && this.sourceNode.node.positions[context]) {
        const sourceNodePosition: Cg2dCoordinate = {
          x: Number(this.sourceNode.node.positions[context].x),
          y: Number(this.sourceNode.node.positions[context].y),
        };

        switch (this.orientation) {
          case SchematicOrientation.Horizontal: {
            return {
              x: sourceNodePosition.x + OUTPUT_SOCKET_OFFSET_HORIZONTAL.x,
              y: sourceNodePosition.y + OUTPUT_SOCKET_OFFSET_HORIZONTAL.y,
            };
          }

          case SchematicOrientation.Vertical: {
            return {
              x: sourceNodePosition.x + OUTPUT_SOCKET_OFFSET_VERTICAL.x,
              y: sourceNodePosition.y + OUTPUT_SOCKET_OFFSET_VERTICAL.y,
            };
          }

          default: {
            return undefined;
          }
        }
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

        switch (this.orientation) {
          case SchematicOrientation.Horizontal: {
            const socketId = this.edge.headVertex.socket;
            const index = _.findIndex(this.inputs(), function(o) {
              return o.name == socketId;
            });

            const positionalOffset = {
              x: INPUT_SOCKET_ALIGNMENT_OFSET.x,
              y: INPUT_SOCKET_ALIGNMENT_OFSET.y * index,
            };
            return {
              x:
                destinationNodePosition.x +
                (INPUT_SOCKET_OFFSET_HORIZONTAL.x + positionalOffset.x),
              y:
                destinationNodePosition.y +
                (INPUT_SOCKET_OFFSET_HORIZONTAL.y + positionalOffset.y),
            };
          }

          case SchematicOrientation.Vertical: {
            return {
              x: destinationNodePosition.x + INPUT_SOCKET_OFFSET_VERTICAL.x,
              y: destinationNodePosition.y + INPUT_SOCKET_OFFSET_VERTICAL.y,
            };
          }

          default: {
            return undefined;
          }
        }
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
      if (
        this.sourceNode &&
        this.destinationNode &&
        this.positionCtx == event.positionCtx
      ) {
        const element = document.getElementById(this.lineId) as HTMLElement;

        /* This sections relies on duplicated code from:
         * - sourceSocketPosition() and destinationSocketPosition().
         * It should be refactored...
         */
        if (element && event.nodeId) {
          switch (this.orientation) {
            case SchematicOrientation.Vertical: {
              if (event.nodeId == this.sourceNode.node.id) {
                const sourceSocketPosition: Cg2dCoordinate = {
                  x: event.nodePosition.x + OUTPUT_SOCKET_OFFSET_VERTICAL.x,
                  y: event.nodePosition.y + OUTPUT_SOCKET_OFFSET_VERTICAL.y,
                };

                element.setAttribute("x1", String(sourceSocketPosition.x));
                element.setAttribute("y1", String(sourceSocketPosition.y));
              } else if (event.nodeId == this.destinationNode.node.id) {
                const destinationSocketPosition: Cg2dCoordinate = {
                  x: event.nodePosition.x + INPUT_SOCKET_OFFSET_VERTICAL.x,
                  y: event.nodePosition.y + INPUT_SOCKET_OFFSET_VERTICAL.y,
                };

                element.setAttribute("x2", String(destinationSocketPosition.x));
                element.setAttribute("y2", String(destinationSocketPosition.y));
              }
              break;
            }

            case SchematicOrientation.Horizontal: {
              const socketId = this.edge.headVertex.socket;
              const index = _.findIndex(this.inputs(), function(o) {
                return o.name == socketId;
              });

              const positionalOffset = {
                x: INPUT_SOCKET_ALIGNMENT_OFSET.x,
                y: INPUT_SOCKET_ALIGNMENT_OFSET.y * index,
              };

              if (event.nodeId == this.sourceNode.node.id) {
                const sourceSocketPosition: Cg2dCoordinate = {
                  x: event.nodePosition.x + OUTPUT_SOCKET_OFFSET_HORIZONTAL.x,
                  y: event.nodePosition.y + OUTPUT_SOCKET_OFFSET_HORIZONTAL.y,
                };
                element.setAttribute("x1", String(sourceSocketPosition.x));
                element.setAttribute("y1", String(sourceSocketPosition.y));
              } else if (event.nodeId == this.destinationNode.node.id) {
                const destinationSocketPosition: Cg2dCoordinate = {
                  x:
                    event.nodePosition.x +
                    (INPUT_SOCKET_OFFSET_HORIZONTAL.x + positionalOffset.x),
                  y:
                    event.nodePosition.y +
                    (INPUT_SOCKET_OFFSET_HORIZONTAL.y + positionalOffset.y),
                };
                element.setAttribute("x2", String(destinationSocketPosition.x));
                element.setAttribute("y2", String(destinationSocketPosition.y));
              }
              break;
            }
          }
        }

        // if (element) {
        //   if (event.nodeId == this.sourceNode.node.id) {
        //     element.setAttribute("x1", String(event.nodePosition.x + OUTPUT_SOCKET_OFFSET_HORIZONTAL.x));
        //     element.setAttribute("y1", String(event.nodePosition.y + OUTPUT_SOCKET_OFFSET_HORIZONTAL.y));

        //   } else if (event.nodeId == this.destinationNode.node.id) {
        //     element.setAttribute("x2", String(event.nodePosition.x + INPUT_SOCKET_OFFSET_HORIZONTAL.x));
        //     element.setAttribute("y2", String(event.nodePosition.y + INPUT_SOCKET_OFFSET_HORIZONTAL.y));
        //   }
        // }
      }
    },
    getNode(nodeId: string): ISchematicNode | undefined {
      if (this.schematic) {
        if (this.schematic.nodes[nodeId]) {
          return this.schematic.nodes[nodeId];
        }
      }
    },
    inputs(): RegistryEntry["inputs"] {
      const sourceNode = this.schematic.nodes[
        this.edge.headVertex.nodeId
      ] as ISchematicNode;

      const sourceNodeEntity = SiEntity.fromJson(sourceNode.object as Entity);

      let inputs = sourceNodeEntity.schema().inputs;

      return inputs;
    },
  },
});
</script>
