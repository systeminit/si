<template>
  <div>
    <div
      class="node absolute cursor-move border-solid border-2 shadow-md"
      :class="nodeIsSelected"
      @mousedown="
        toggleSelection(true);
        selectNode();
      "
      @contextmenu="contextMenu($event)"
    >
      <div class="flex flex-col select-none">
        <div class="flex flex-col text-white ml-1 mt-1">
          <div class="font-light text-xs">name:</div>
          <div
            class="font-normal text-xs ml-2 text-red-700"
            v-if="nodeObject.deleted"
          >
            {{ nodeObject.name }}
          </div>
          <div class="font-normal text-xs ml-2" v-else>
            {{ nodeObject.name }}
          </div>
          <span v-if="nodeObject.changeSetId" class="text-xs">
            <span class="font-light">changeSet:</span>
            <div class="ml-2">
              {{ changeSetName(nodeObject.changeSetId) }}
            </div>
          </span>
        </div>
      </div>
      <!-- 
      <vue-simple-context-menu
        :elementId="'myFirstMenu'"
        :options="optionsArray1"
        :ref="'vueSimpleContextMenu1'"
        @option-clicked="optionClicked1"
      /> -->
    </div>
  </div>
</template>

<script>
import { registry } from "si-registry";
import { mapState, mapActions } from "vuex";

export default {
  name: "NodeObject",
  props: {
    nodeObject: {},
  },
  data() {
    return {
      // entityId: this.nodeObject.id,
      // entityName: this.nodeObject.name,
      isSelected: false,
      optionsArray1: [
        {
          name: "Duplicate",
          slug: "duplicate",
        },
        {
          name: "Edit",
          slug: "edit",
        },
        {
          name: "Delete",
          slug: "delete",
        },
      ],
    };
  },
  methods: {
    // ...mapActions('editor', ['selectNode']),
    selectNode() {
      this.$store.dispatch("editor/selectNode", this.nodeObject);
    },
    toggleSelection(value) {
      this.isSelected = value;
    },
    contextMenu(e) {
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
    nodeIsSelected: function() {
      return {
        "node-is-selected": this.isSelected,
      };
    },
    ...mapState({
      selectedNode: state => state.editor.selectedNode,
    }),
  },
  watch: {
    selectedNode(newState, previousState) {
      if (newState.id != this.nodeObject.id) {
        this.toggleSelection(false);
      }
    },
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
