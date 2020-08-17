<template>
  <div>
    <svg height="100%" width="100%" class="absolute">
      <line
        :x1="sourceSocketPosition.x"
        :y1="sourceSocketPosition.y"
        :x2="destinationSocketPosition.x"
        :y2="destinationSocketPosition.y"
        style="stroke:rgb(71,99,113); stroke-width:2"
      />
    </svg>
  </div>
</template>

<script>
/**
 * - Connections between nodes.
 *  We should have a connectionList component that is a single SVG with a line per connection.
 *  A connection should take a sourceNodeSocket and destinationNodeSocket
 *  We should extract the head and tail position from the sockets
 *
 *  This proto is simulating some of that and will need to be refactored.
 *
 */

import { mapGetters } from "vuex";
export default {
  name: "ConnectionObject",
  props: {
    sourceNodePosition: Object,
    destinationNodePosition: Object,
    // sourceNode: {}, //refactor to sourceSocket: {},
    // destinationNode: {}, // refactor to destinationSocket: {},
  },
  computed: {
    // We get the position of the source socket
    sourceSocketPosition() {
      // the node list get updated so I can't just pull node[i]
      //let nodeId = "node:3c235077-69bc-4416-99d9-610bf3c1a9fd"; // should be sourceNode.id
      //const node = this.$store.getters["node/getNodebyId"](nodeId);

      const outputSocketOffset = {
        x: 68.5, // node center.
        y: 100, // bottom line of a node
      };

      return {
        x: this.sourceNodePosition.x + outputSocketOffset.x,
        y: this.sourceNodePosition.y + outputSocketOffset.y,
      };
    },
    // We get the position of the destination socket
    destinationSocketPosition() {
      // the node list get updated so I can't just pull node[i]
      //let nodeId = "node:4f004a6c-b4d3-48a8-b11e-4d31b5e3c5f0"; // should be destinationNode.id
      //const node = this.$store.getters["node/getNodebyId"](nodeId);

      const inputSocketOffset = {
        x: 68.5, // node center.
        y: 0, // top line of a node
      };

      return {
        x: this.destinationNodePosition.x + inputSocketOffset.x,
        y: this.destinationNodePosition.y + inputSocketOffset.y,
      };
    },
  },
};
//226, 340
</script>

<style>
.line-segment {
  background-color: blue;
  bottom: var(--y);
  height: 3px;
  left: var(--x);
  position: absolute;
  transform: rotate(calc(var(--angle) * 1deg));
  transform-origin: left bottom;
  width: calc(var(--hypotenuse) * 1px);
}
</style>
