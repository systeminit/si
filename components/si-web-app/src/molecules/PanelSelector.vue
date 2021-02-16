<template>
  <div
    class="flex flex-row w-full h-full"
    @mouseenter="activateShortcuts()"
    @mouseleave="deactivateShortcuts()"
  >
    <EmptyPanel
      :panelRef="panelRef"
      :panelContainerRef="panelContainerRef"
      @change-panel="changePanelType"
      @panel-maximized-full="setMaximizedFull($event)"
      @panel-maximized-container="setMaximizedContainer($event)"
      @panel-minimized-full="setMaximizedFull($event)"
      @panel-minimized-container="setMaximizedContainer($event)"
      :initialMaximizedFull="maximizedFull"
      :initialMaximizedContainer="maximizedContainer"
      v-if="panelType == 'empty'"
    />
    <SystemSchematicPanel
      :panelRef="panelRef"
      :panelContainerRef="panelContainerRef"
      @change-panel="changePanelType"
      @panel-maximized-full="setMaximizedFull($event)"
      @panel-maximized-container="setMaximizedContainer($event)"
      @panel-minimized-full="setMaximizedFull($event)"
      @panel-minimized-container="setMaximizedContainer($event)"
      :initialMaximizedFull="maximizedFull"
      :initialMaximizedContainer="maximizedContainer"
      v-else-if="panelType == 'systemSchematic'"
    />
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import EmptyPanel from "@/organisims/EmptyPanel.vue";
import SystemSchematicPanel from "@/organisims/SystemSchematicPanel.vue";
import { PanelEventBus } from "@/atoms/PanelEventBus";

export enum PanelType {
  Empty = "empty",
  SystemSchematic = "systemSchematic",
}

export interface IData {
  shortcuts: boolean;
  panelType: PanelType;
  maximizedFull: boolean;
  maximizedContainer: boolean;
}

export default Vue.extend({
  name: "PanelSelector",
  props: {
    panelRef: String,
    panelContainerRef: String,
    initialPanelType: {
      type: String as PropType<PanelType>,
      default: PanelType.Empty,
    },
  },
  components: {
    EmptyPanel,
    SystemSchematicPanel,
  },
  data(): IData {
    return {
      shortcuts: false,
      panelType: this.initialPanelType,
      maximizedFull: false,
      maximizedContainer: false,
    };
  },
  mounted() {
    PanelEventBus.$on("shortcut", this.handleShortcut);
  },
  beforeDestroy() {
    PanelEventBus.$off("shortcut", this.handleShortcut);
  },
  methods: {
    activateShortcuts() {
      this.shortcuts = true;
    },
    deactivateShortcuts() {
      this.shortcuts = false;
    },
    handleShortcut(event: KeyboardEvent) {
      if (this.shortcuts) {
        if (event.altKey && event.shiftKey && event.key == "N") {
          PanelEventBus.$emit("create-new-panel", {
            panelContainerRef: this.panelContainerRef,
          });
        }
        if (event.altKey && event.shiftKey && event.key == "D") {
          PanelEventBus.$emit("delete-panel", {
            panelRef: this.panelRef,
          });
        }
      }
    },
    setMaximizedFull(to: boolean) {
      this.maximizedFull = to;
    },
    setMaximizedContainer(to: boolean) {
      this.maximizedContainer = to;
    },
    changePanelType(newPanelType: PanelType) {
      this.panelType = newPanelType;
    },
  },
});
</script>
