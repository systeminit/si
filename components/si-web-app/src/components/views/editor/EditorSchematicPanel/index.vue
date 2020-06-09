<template>
  <!-- eslint-disable vue/no-unused-components -->
  <div ref="schematic-panel" class="flex flex-col h-full w-full">
      
    <div id="schematic-panel-menu" class="flex flex-row justify-between flex-no-wrap content-between bg-black w-full">
      
      <div class="flex flex-row justify-start mx-3">      
        
        <button class="text-white px-4 py-2 focus:outline-none" @click="addNode()" type="button">
          <plus-square-icon size="1.1x"/>
        </button>

      </div>

      <div class="mx-3">
        <button class="text-white px-4 py-2 focus:outline-none">
          <settings-icon size="1.1x"/>
        </button>

        <button class="text-white px-4 py-2 focus:outline-none" @click="maximizePanel()" type="button">
          <maximize-2-icon size="1.1x"></maximize-2-icon>
        </button>
      </div>
    
    </div>

    <div class="flex w-full h-full">
      <NodeEditor/>
    </div>
  </div>
</template>

<script>
  /* eslint-disable vue/no-unused-components */
import {
  Maximize2Icon,
  SettingsIcon,
  FilterIcon,
  SearchIcon,
  CodeIcon,
  PlusSquareIcon,
} from "vue-feather-icons";
import UiButton from "@/components/ui/button/UiButton.vue";
import NodeEditor from "./NodeEditor.vue";

export default {
  name: "EditorSchematicPanel",
  components: {
    Maximize2Icon,
    SettingsIcon,
    FilterIcon,
    SearchIcon,
    CodeIcon,
    UiButton,
    NodeEditor,
    PlusSquareIcon,
  },
  methods: {
    maximizePanel() {
      this.$emit("maximizePanelMsg", {
        panel: {
          id: "schematic"
        },
      })
    },
    addNode() {
      let payload = {
        id: Date.now().toString(),
        name: "new",
        isEntity:false
      }
      this.$store.dispatch('editor/addNode', payload)
    }
  }
};
</script>

<style scoped>
.property-editor-bg-color {
  background-color: #212324;
}

.property-title-bg-color {
  background-color: #292C2D;
}

.is-hidden .schematic-editor {
  @apply overflow-hidden h-0;
}


</style>