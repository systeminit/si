<template>
  <div ref="property-panel" class="h-full w-full property-editor-bg-color">


    <div id="property-panel-menu" class="flex flex-row justify-between flex-no-wrap content-between bg-black w-full">
      
      <div class="flex flex-row justify-start mx-3">      
        
        <button class="text-white px-4 py-2 focus:outline-none">
          <search-icon size="1.1x"/>
        </button>

        <button class="text-white px-4 py-2 focus:outline-none">
          <filter-icon size="1.1x"/>
        </button>

        <button class="text-white px-4 py-2 focus:outline-none">
          <code-icon size="1.1x"/>
        </button>
      </div>

      <div class="mx-3">
        <button class="text-white px-4 py-2 focus:outline-none" @click="maximizePanel()" type="button">
          <maximize-2-icon size="1x"></maximize-2-icon>
        </button>
      </div>
    
    </div>

    <div class="text-red-600">
      {{selectedNodeId}}
    </div>

    <div class="flex w-full h-full overflow-auto mt-5">
      <PropertyListView
        :nodeId="nodeId"
      />

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
import PropertyListView from "./PropertyListView.vue";
import { mapState, mapActions } from 'vuex'

export default {
  name: "EditorPropertyPanel",
  components: {
    Maximize2Icon,
    FilterIcon,
    SearchIcon,
    CodeIcon,
    PropertyListView,
  },
  data() {
    let mode = "view"
    // let nodeId = this.selectedNodeId
    let nodeId = "kubernetes_deployment_entity:f17c2635-ce32-4a17-857d-033d68b62ba7"
    return {
      mode,
      nodeId
    }
  },
  methods: {
    maximizePanel() {
      this.$emit("maximizePanelMsg", {
        panel: {
          id: "property"
        },
      })
    }
  },
  computed: mapState({
    selectedNodeId: state => state.editor.selectedNodeId
  }),
};
</script>

<style scoped>
.property-editor-bg-color {
  background-color: #212324;
}

.property-title-bg-color {
  background-color: #292C2D;
}

</style>