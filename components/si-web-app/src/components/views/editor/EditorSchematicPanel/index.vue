<template>
  <!-- eslint-disable vue/no-unused-components -->
  <div ref="schematic-panel" class="flex flex-col h-full w-full">

    <div id="schematic-panel-menu" class="flex flex-row flex-no-wrap content-between bg-black h-30 w-full">
      
      <ui-button kind="icon">
        <filter-icon size="1x" class="custom-class"></filter-icon>
      </ui-button>

      <ui-button kind="icon">
        <code-icon size="1x" class="custom-class"></code-icon>
      </ui-button>

      <ui-button kind="icon">
        <search-icon size="1x" class="custom-class"></search-icon>
      </ui-button>

      <ui-button kind="icon">
        <filter-icon size="1x" class="custom-class"></filter-icon>
      </ui-button>

      <ui-button kind="icon">
        <settings-icon size="1x" class="custom-class"></settings-icon>
      </ui-button>

      <ui-button kind="icon">
        <maximize-2-icon size="1x" class="custom-class"></maximize-2-icon>
      </ui-button>
    
    </div>

    <div class="flex w-full h-full overflow-auto">
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
  },
  data() {
    return {
      isVisible: true
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