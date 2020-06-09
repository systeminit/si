<template>
  <!-- eslint-disable vue/no-unused-components -->
  <div>
    <div class="node absolute cursor-move border-solid border-2 shadow-md" :class="nodeIsSelected" @mousedown="toggleSelection(true); selectNode()">
      <div class="flex flex-col select-none">

        <div class="flex flex-col text-white ml-1 mt-1">
          <div class="font-light text-xs">name:</div>
          <div class="font-normal text-xs ml-2">{{nodeObject.name}}</div>
        </div>
      
      </div>

    </div>
  </div>
</template>

<script>
import { registry } from "si-registry";
import { mapState, mapActions } from 'vuex'

export default {
  name: "NodeObject",
  props: {
    nodeObject: {},
  },
  data() {
    return {
      // entityId: this.nodeObject.id,
      // entityName: this.nodeObject.name,
      isSelected: false
    };
  },
  methods: {
  // ...mapActions('editor', ['selectNode']),
    selectNode() {
      this.$store.dispatch('editor/selectNode', this.nodeObject)

    },
    toggleSelection(value) {
      this.isSelected = value;
    },
  },
  computed: {
    nodeIsSelected: function() {
      return {
        'node-is-selected': this.isSelected,
      };
    },
    ...mapState({
      selectedNode: state => state.editor.selectedNode
    }),
  },
  watch: {
    selectedNode (newState, previousState) {
      console.log("new state:", newState)
      if (newState.id != this.nodeObject.id) {
        this.toggleSelection(false)
      }
    }
  }
}
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
  border-color: #00B0B1;
}
</style>