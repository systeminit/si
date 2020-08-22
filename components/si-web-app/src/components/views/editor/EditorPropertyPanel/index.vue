<template>
  <div ref="property-panel" class="w-full h-full property-editor-bg-color">
    <div
      id="property-panel-menu"
      class="flex flex-row flex-no-wrap content-between justify-between w-full bg-black"
    >
      <div class="flex flex-row justify-start mx-3">
        <button class="px-4 py-2 text-white focus:outline-none">
          <search-icon size="1.1x" />
        </button>

        <button class="px-4 py-2 text-white focus:outline-none">
          <filter-icon size="1.1x" />
        </button>

        <button
          class="px-4 py-2 text-white focus:outline-none"
          @click="toggleCodeEditor"
        >
          <code-icon size="1.1x" />
        </button>
      </div>

      <div class="mx-3">
        <button
          class="px-4 py-2 text-white focus:outline-none"
          @click="maximizePanel()"
          type="button"
        >
          <maximize-2-icon size="1x"></maximize-2-icon>
        </button>
      </div>
    </div>

    <Monaco v-if="codeEditor" />
    <PropertyList v-else />
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import {
  Maximize2Icon,
  SettingsIcon,
  FilterIcon,
  SearchIcon,
  CodeIcon,
} from "vue-feather-icons";
import Monaco from "@/components/ui/Monaco.vue";

import PropertyList from "./PropertyList.vue";

import { mapState, mapActions } from "vuex";

export default Vue.extend({
  name: "EditorPropertyPanel",
  components: {
    Maximize2Icon,
    FilterIcon,
    SearchIcon,
    CodeIcon,
    PropertyList,
    Monaco,
  },
  data() {
    return {
      codeEditor: false,
    };
  },
  methods: {
    toggleCodeEditor(): void {
      this.codeEditor = !this.codeEditor;
    },
    maximizePanel(): void {
      this.$emit("maximizePanelMsg", {
        panel: {
          id: "property",
        },
      });
    },
  },
});
</script>

<style scoped>
.property-editor-bg-color {
  background-color: #212324;
}

.property-title-bg-color {
  background-color: #292c2d;
}
</style>
