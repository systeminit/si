<template>
  <div>
    <div
      class="absolute shadow-md cursor-move node-container node"
      :id="node.id"
      :class="nodeIsSelected"
      v-bind:style="positionStyle"
      :data-cy="'editor-schematic-panel-node-list-' + nodeIndex"
      :data-cy-name="
        'editor-schematic-panel-node-list-' +
        node.objectType +
        '-' +
        displayItem.name
      "
      @mousedown="selectNode()"
    >
      <span
        :ref="`${node.id}.socket.input`"
        :id="`${node.id}.socket.input`"
        class="socket-input socket node"
      />
      <span
        :ref="`${node.id}.socket.output`"
        :id="`${node.id}.socket.output`"
        class="socket-output socket node"
      />

      <div class="flex flex-col select-none node">
        <div class="flex flex-col text-white node">
          <div class="node-title-bar node">
            <div
              class="mt-1 text-xs font-medium text-center node"
              :class="nodeTitleClasses"
            >
              {{ displayItem.objectType }}
            </div>
          </div>
          <div
            class="mt-2 text-xs font-normal text-center text-red-700 node"
            v-if="displayItem.siStorable.deleted"
          >
            {{ displayItem.name }}
          </div>
          <div class="mt-2 text-xs font-normal text-center node" v-else>
            {{ displayItem.name }}
          </div>
          <span
            v-if="displayItem.head == false && currentChangeSet"
            class="text-xs node"
          >
            <span class="font-light node">changeSet:</span>
            <div class="ml-2 node">
              {{ currentChangeSet.name }}
            </div>
          </span>
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

import { RootStore } from "@/store";
import { Node, NodeObject } from "@/api/sdf/model/node";

export default Vue.extend({
  name: "NodeObject",
  props: {
    node: {
      type: Object as PropType<Node>,
    },
    nodeIndex: {
      type: Number,
    },
  },
  methods: {
    async selectNode() {
      await this.$store.dispatch("editor/node", this.node);
    },
  },
  computed: {
    positionStyle(): Record<string, string> {
      let editorContext = this.$store.state.editor.context;
      let position = Node.upgrade(this.node).position(editorContext);
      return {
        left: `${position.x}px`,
        top: `${position.y}px`,
      };
    },
    nodeIsSelected(): Record<string, boolean> {
      if (
        this.node &&
        this.selectedNode &&
        this.selectedNode.id == this.node.id
      ) {
        return {
          "node-is-selected": true,
        };
      } else {
        return {
          "node-is-selected": false,
        };
      }
    },
    displayItem(): NodeObject {
      return this.editObject;
    },
    nodeTitleClasses(): Record<string, boolean> {
      if (this.currentChangeSet) {
        if (this.displayItem.head == false) {
          return {
            "input-border-gold": true,
            border: true,
            "border-t-0": true,
            "border-b-2": true,
            "border-r-0": true,
            "border-l-0": true,
          };
        }
      }
      return {};
    },
    ...mapState({
      editObject(state: RootStore) {
        return state.editor.objects[this.node.id];
      },
      selectedNode(state: RootStore) {
        return state.editor.node;
      },
      currentChangeSet(state: RootStore) {
        return state.editor.changeSet;
      },
    }),
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
  @apply z-10;
  border-radius: 6px;
  border-width: 1px;
  border-color: #5cb1b1;
}
</style>
