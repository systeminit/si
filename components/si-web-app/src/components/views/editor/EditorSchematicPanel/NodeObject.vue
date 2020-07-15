<template>
  <div>
    <div
      class="node absolute cursor-move border-solid border-2 shadow-md"
      :class="nodeIsSelected"
      @mousedown="selectNode()"
      @contextmenu="contextMenu($event)"
    >
      <div class="flex flex-col select-none">
        <div class="flex flex-col text-white ml-1 mt-1">
          <div class="text-xs">
            {{ displayItem.siStorable.typeName }}
          </div>
          <div class="font-light text-xs">name:</div>
          <div
            class="font-normal text-xs ml-2 text-red-700"
            v-if="displayItem.siStorable.deleted"
          >
            {{ nodeObject.name }}
          </div>
          <div class="font-normal text-xs ml-2" v-else>
            {{ nodeObject.name }}
          </div>
          <span v-if="displayItem.siStorable.changeSetId" class="text-xs">
            <span class="font-light">changeSet:</span>
            <div class="ml-2">
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
  name: "NodeObject",
  props: {
    nodeObject: {},
  },
  methods: {
    selectNode() {
      this.$store.dispatch("node/current", { node: this.nodeObject });
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
// fullstack-schema.graphql
// NodeObject.vue
</script>

<style type="text/css" scoped>
.node {
  width: 140px;
  height: 100px;
  background-color: teal;
  color: #fff;
  border-color: teal;
}

.node-is-selected {
  /*@apply border-2;*/
  @apply z-10;
  border-color: #00b0b1;
}
</style>
