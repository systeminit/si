<template>
  <!-- eslint-disable vue/no-unused-components -->
  <div>
    <div :ref="entityId" class="node absolute cursor-move border-solid" :class="nodeIsSelected" @mousedown="toggleSelection(true); selectNode(entityId)">
      <div class="flex flex-col select-none">

        <div class="flex flex-col text-white ml-1 mt-1">
          <div class="font-light text-xs">name:</div>
          <div class="font-normal text-xs ml-2">{{entityName}}</div>
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
      entityId: this.nodeObject.id,
      entityName: this.nodeObject.name,
      isSelected: false
    };
  },
  methods: {
  ...mapActions('editor', ['selectNode']),

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
      selectedNodeId: state => state.editor.selectedNodeId
    }),
  },
  watch: {
    selectedNodeId (newState, previousState) {
      if (newState != this.nodeObject.id) {
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
}

.node-is-selected {
  @apply border-2;
  
  border-color: #00B0B1;
}
</style>