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
    // sourceNode: {}, //refactor to sourceSocket: {},
    // destinationNode: {}, // refactor to destinationSocket: {},
  },
  computed: {
    // We get the position of the source socket
    sourceSocketPosition() {
      try {
        // the node list get updated so I can't just pull node[i]
        let nodeId = "system_entity:24a30b19-a3be-4230-a89b-9100fc09b155"; // should be sourceNode.id
        const node = this.$store.getters["node/getNodebyId"](nodeId);

        const outputSocketOffset = {
          x: 68.5, // node center.
          y: 100, // bottom line of a node
        };

        if ("position" in node) {
          return {
            x: node.position.x + outputSocketOffset.x,
            y: node.position.y + outputSocketOffset.y,
          };
        } else {
          return {
            x: "397",
            y: "77",
          };
        }
      } catch(err) {
        return {
            x: "397",
            y: "77",
          };
      }
    },
    // We get the position of the destination socket
    destinationSocketPosition() {
      try {
        // the node list get updated so I can't just pull node[i]
        let nodeId = "application_entity:9d06874f-222c-4ede-9873-f61bdbc9b1ad"; // should be destinationNode.id
        const node = this.$store.getters["node/getNodebyId"](nodeId);

        const inputSocketOffset = {
          x: 68.5, // node center.
          y: 0, // top line of a node
        };

        if ("position" in node) {
          return {
            x: node.position.x + inputSocketOffset.x,
            y: node.position.y + inputSocketOffset.y,
          };
        } else {
          return {
            x: "100",
            y: "100",
          };
        }
      } catch(err) {
          return {
            x: "100",
            y: "100",
          };
      }
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
