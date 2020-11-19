<template>
  <div ref="property-panel" class="w-full h-full property-editor-bg-color">
    <div
      id="property-panel-menu"
      class="flex flex-row flex-no-wrap content-between justify-between w-full bg-black"
    >
      <div class="flex flex-row justify-start mx-3">
        <button
          class="px-4 py-2 text-white focus:outline-none"
          :class="{ activeMenuItem: isViewActive('propertyView') }"
          @click="setActiveView('propertyView')"
        >
          <disc-icon size="1.1x" />
        </button>

        <button
          class="px-4 py-2 text-white focus:outline-none"
          :class="{ activeMenuItem: isViewActive('codeView') }"
          @click="setActiveView('codeView')"
        >
          <code-icon size="1.1x" />
        </button>

        <!-- FIX ME: Only show the code button if the node has "native configs that can be viewed as code" -->
        <button
          class="px-4 py-2 text-white focus:outline-none"
          :class="{ activeMenuItem: isViewActive('eventView') }"
          @click="setActiveView('eventView')"
        >
          <radio-icon size="1.1x" />
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

    <div class="relative w-full h-full property-panel-view">
      <CodeViewer v-show="isViewActive('codeView')" :key="selectedNode.id" />
      <PropertyViewer
        v-show="isViewActive('propertyView')"
        :selectedNode="selectedNode"
      />
      <EventViewer
        v-show="isViewActive('eventView')"
        :selectedNode="selectedNode"
      />
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { mapState } from "vuex";
import {
  Maximize2Icon,
  SettingsIcon,
  CodeIcon,
  DiscIcon,
  RadioIcon,
} from "vue-feather-icons";

import CodeViewer from "./CodeViewer.vue";

// @ts-ignore
import PropertyViewer from "./PropertyViewer";
// @ts-ignore
import EventViewer from "./EventViewer";

import { RegistryProperty, Node } from "@/api/sdf/model/node";

export default Vue.extend({
  name: "EditorPropertyPanel",
  components: {
    Maximize2Icon,
    CodeIcon,
    DiscIcon,
    RadioIcon,
    PropertyViewer,
    CodeViewer,
    EventViewer,
  },
  data() {
    return {
      activeView: "propertyView",
    };
  },
  computed: {
    ...mapState({
      selectedNode: (state: any): Node => state.editor.node,
    }),
  },
  methods: {
    setActiveView(view: string): void {
      this.activeView = view;
    },
    isViewActive(view: string): boolean {
      return this.activeView === view;
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

.activeMenuItem {
  color: #b7e7ef;
}
</style>
