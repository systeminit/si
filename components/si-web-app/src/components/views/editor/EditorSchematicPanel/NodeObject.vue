<template>
  <div>
    <div
      class="absolute shadow-md cursor-move node-container node"
      :id="nodeObject.id"
      :class="nodeIsSelected"
      v-bind:style="positionStyle"
      @mousedown="selectNode()"
      @contextmenu="contextMenu($event)"
    >
      <span
        ref="`${nodeObject.id}.socket.input`"
        class="socket-input node"
        @mousedown="selectSocket($event)"
      />
      <span
        ref="`${nodeObject.id}.socket.output`"
        class="socket-output node"
        @mousedown="selectSocket($event)"
      />

      <div class="flex flex-col select-none node">
        <div class="flex flex-col text-white node">
          <div class="node-title-bar node">
            <div class="mt-1 text-xs font-medium text-center node">
              {{ displayItem.siStorable.typeName.split("_")[0] }}
            </div>
          </div>
          <div
            class="mt-2 text-xs font-normal text-center text-red-700 node"
            v-if="displayItem.siStorable.deleted"
          >
            {{ nodeObject.name }}
          </div>
          <div class="mt-2 text-xs font-normal text-center node" v-else>
            {{ nodeObject.name }}
          </div>
          <span v-if="displayItem.siStorable.changeSetId" class="text-xs node">
            <span class="font-light node">changeSet:</span>
            <div class="ml-2 node">
              {{ changeSetName(displayItem.siStorable.changeSetId) }}
            </div>
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { registry } from "si-registry";
import { mapState, mapActions } from "vuex";
import _ from "lodash";

export default {
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

  name: "NodeObject",
  props: {
    nodeObject: {},
  },
  methods: {
    selectNode() {
      this.$store.dispatch("node/current", { node: this.nodeObject });
    },
    selectSocket(event) {
      console.log("socket");
      console.log(event);
      console.log(this.nodeObject.id + ".socket.input");
    },
    ontextMenu(e) {
      e.preventDefault();
      this.$refs.vueSimpleContextMenu1.showMenu(event, null);
    },
    optionClicked1(event) {
      window.alert(JSON.stringify(event));
    },
    changeSetName(changeSetId) {
      if (changeSetId) {
        const changeSet = this.$store.getters["changeSet/byId"](changeSetId);
        return changeSet.name;
      } else {
        return "";
      }
    },
  },
  computed: {
    positionStyle() {
      return {
        left: `${this.nodeObject.position.x}px`,
        top: `${this.nodeObject.position.y}px`,
      };
    },
    nodeIsSelected() {
      if (
        this.nodeObject &&
        this.selectedNode &&
        this.selectedNode.id == this.nodeObject.id
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
    displayItem() {
      let node = _.find(this.$store.state.node.nodes, [
        "id",
        this.nodeObject.id,
      ]);
      if (this.currentChangeSet && node.display[this.currentChangeSet.id]) {
        return node.display[this.currentChangeSet.id];
      } else {
        return node.display["saved"];
      }
    },
    ...mapState({
      selectedNode: state => state.node.current,
      currentChangeSet: state => state.changeSet.current,
    }),
  },
};
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
