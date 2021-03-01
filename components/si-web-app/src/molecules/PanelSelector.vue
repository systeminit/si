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
    <AttributePanel
      :panelRef="panelRef"
      :panelContainerRef="panelContainerRef"
      @change-panel="changePanelType"
      @panel-maximized-full="setMaximizedFull($event)"
      @panel-maximized-container="setMaximizedContainer($event)"
      @panel-minimized-full="setMaximizedFull($event)"
      @panel-minimized-container="setMaximizedContainer($event)"
      :initialMaximizedFull="maximizedFull"
      :initialMaximizedContainer="maximizedContainer"
      v-if="panelType == 'attribute'"
    />
    <SecretPanel
      :panelRef="panelRef"
      :panelContainerRef="panelContainerRef"
      @change-panel="changePanelType"
      @panel-maximized-full="setMaximizedFull($event)"
      @panel-maximized-container="setMaximizedContainer($event)"
      @panel-minimized-full="setMaximizedFull($event)"
      @panel-minimized-container="setMaximizedContainer($event)"
      :initialMaximizedFull="maximizedFull"
      :initialMaximizedContainer="maximizedContainer"
      v-else-if="panelType == 'secret'"
    />
    <SchematicPanel
      :panelRef="panelRef"
      :panelContainerRef="panelContainerRef"
      @change-panel="changePanelType"
      @panel-maximized-full="setMaximizedFull($event)"
      @panel-maximized-container="setMaximizedContainer($event)"
      @panel-minimized-full="setMaximizedFull($event)"
      @panel-minimized-container="setMaximizedContainer($event)"
      :initialMaximizedFull="maximizedFull"
      :initialMaximizedContainer="maximizedContainer"
      v-else-if="panelType == 'schematic'"
    />
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import EmptyPanel from "@/organisims/EmptyPanel.vue";
import SecretPanel from "@/organisims/SecretPanel.vue";
import AttributePanel from "@/organisims/AttributePanel.vue";
import SchematicPanel from "@/organisims/SchematicPanel.vue";
import { PanelEventBus } from "@/atoms/PanelEventBus";
import Bottle from "bottlejs";
import { Persister } from "@/api/persister";

export enum PanelType {
  Empty = "empty",
  Attribute = "attribute",
  Secret = "secret",
  Schematic = "schematic",
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
    SecretPanel,
    AttributePanel,
    SchematicPanel,
  },
  data(): IData {
    let bottle = Bottle.pop("default");
    let persister: Persister = bottle.container.Persister;
    let savedData = persister.getData(this.panelRef);
    if (savedData) {
      return savedData;
    } else {
      return {
        shortcuts: false,
        panelType: this.initialPanelType,
        maximizedFull: false,
        maximizedContainer: false,
      };
    }
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
  watch: {
    $data: {
      handler: function(newData, oldData) {
        let bottle = Bottle.pop("default");
        let persister: Persister = bottle.container.Persister;
        persister.setData(this.panelRef, newData);
      },
      deep: true,
    },
  },
});
</script>
