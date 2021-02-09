<template>
  <div id="schematic-panel" class="w-full h-full grid-background">
    <div
      ref="canvasParent"
      data-cy="editor-schematic-panel-node-editor-canvas"
      class="w-full h-full overflow-hidden"
    >
      <div
        ref="canvas"
        id="canvas"
        class="relative flex-auto block w-full h-full canvas"
      >
        <svg
          id="transientConnectionSvg"
          height="100%"
          width="100%"
          class="absolute"
        />

        <SiGrid class="z-10" />
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";

import SiGrid from "@/atoms/SiGrid.vue";

export default Vue.extend({
  name: "SchematicPanel",
  components: {
    SiGrid,
  },
});
</script>
