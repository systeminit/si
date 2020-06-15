<template>
  <div ref="property-panel" class="h-full w-full property-editor-bg-color">
    <div
      id="property-panel-menu"
      class="flex flex-row justify-between flex-no-wrap content-between bg-black w-full"
    >
      <div class="flex flex-row justify-start mx-3">
        <button class="text-white px-4 py-2 focus:outline-none">
          <search-icon size="1.1x" />
        </button>

        <button class="text-white px-4 py-2 focus:outline-none">
          <filter-icon size="1.1x" />
        </button>

        <button class="text-white px-4 py-2 focus:outline-none">
          <code-icon size="1.1x" />
        </button>
      </div>

      <div class="mx-3">
        <button
          class="text-white px-4 py-2 focus:outline-none"
          @click="maximizePanel()"
          type="button"
        >
          <maximize-2-icon size="1x"></maximize-2-icon>
        </button>
      </div>
    </div>

    <div v-if="selectedNode">
      <div v-if="selectedNode.name === 'new'">
        <div class="flex w-full h-full mt-5 overflow-auto">
          <PropertyListCreate :node="selectedNode" />
        </div>
      </div>

      <div v-else>
        <div class="flex w-full h-fullmt-5 overflow-auto">
          <PropertyListView :nodeId="selectedNode.id" />
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import {
  Maximize2Icon,
  SettingsIcon,
  FilterIcon,
  SearchIcon,
  CodeIcon,
} from "vue-feather-icons";
import PropertyListView from "./viewMode/PropertyListView.vue";
import PropertyListCreate from "./createMode/PropertyListCreate.vue";

import { mapState, mapActions } from "vuex";

export default {
  name: "EditorPropertyPanel",
  components: {
    Maximize2Icon,
    FilterIcon,
    SearchIcon,
    CodeIcon,
    PropertyListView,
    PropertyListCreate,
  },
  data() {
    return {
      // selectedNode: {}
      // selectedNode
    };
  },
  // watch: {
  //   selectedNode (newState, previousState) {
  //     this.selectedNode = newState
  //   }
  // },
  methods: {
    maximizePanel() {
      this.$emit("maximizePanelMsg", {
        panel: {
          id: "property",
        },
      });
    },
  },
  computed: mapState({
    selectedNode: state => state.editor.selectedNode,
  }),
};
</script>

<style scoped>
.property-editor-bg-color {
  background-color: #212324;
}

.property-title-bg-color {
  background-color: #292c2d;
}
</style>
