<template>
  <div
    class="flex flex-row w-full h-full"
    @mouseenter="activateShortcuts()"
    @mouseleave="deactivateShortcuts()"
  >
    <AttributePanel
      :isVisible="isVisible"
      :panelIndex="panelIndex"
      :panelRef="panelRef"
      :panelContainerRef="panelContainerRef"
      @change-panel="changePanelType"
      @panel-maximized-full="setMaximizedFull($event)"
      @panel-maximized-container="setMaximizedContainer($event)"
      @panel-minimized-full="setMaximizedFull($event)"
      @panel-minimized-container="setMaximizedContainer($event)"
      :initialMaximizedFull="maximizedFull"
      :initialMaximizedContainer="maximizedContainer"
      :isMaximizedContainerEnabled="isMaximizedContainerEnabled"
      v-if="panelType == 'attribute'"
    />
    <SecretPanel
      :isVisible="isVisible"
      :panelIndex="panelIndex"
      :panelRef="panelRef"
      :panelContainerRef="panelContainerRef"
      @change-panel="changePanelType"
      @panel-maximized-full="setMaximizedFull($event)"
      @panel-maximized-container="setMaximizedContainer($event)"
      @panel-minimized-full="setMaximizedFull($event)"
      @panel-minimized-container="setMaximizedContainer($event)"
      :initialMaximizedFull="maximizedFull"
      :initialMaximizedContainer="maximizedContainer"
      :isMaximizedContainerEnabled="isMaximizedContainerEnabled"
      v-else-if="panelType == 'secret'"
    />
    <SchematicPanel
      :isVisible="isVisible"
      :panelIndex="panelIndex"
      :panelRef="panelRef"
      :panelContainerRef="panelContainerRef"
      @change-panel="changePanelType"
      @panel-maximized-full="setMaximizedFull($event)"
      @panel-maximized-container="setMaximizedContainer($event)"
      @panel-minimized-full="setMaximizedFull($event)"
      @panel-minimized-container="setMaximizedContainer($event)"
      :initialMaximizedFull="maximizedFull"
      :initialMaximizedContainer="maximizedContainer"
      :isMaximizedContainerEnabled="isMaximizedContainerEnabled"
      v-else-if="panelType == 'schematic'"
    />
    <WorkflowPanel
      :isVisible="isVisible"
      :panelIndex="panelIndex"
      :panelRef="panelRef"
      :panelContainerRef="panelContainerRef"
      @change-panel="changePanelType"
      @panel-maximized-full="setMaximizedFull($event)"
      @panel-maximized-container="setMaximizedContainer($event)"
      @panel-minimized-full="setMaximizedFull($event)"
      @panel-minimized-container="setMaximizedContainer($event)"
      :initialMaximizedFull="maximizedFull"
      :initialMaximizedContainer="maximizedContainer"
      :isMaximizedContainerEnabled="isMaximizedContainerEnabled"
      v-else-if="panelType == 'workflow'"
    />
    <ActionPanel
      :isVisible="isVisible"
      :panelIndex="panelIndex"
      :panelRef="panelRef"
      :panelContainerRef="panelContainerRef"
      @change-panel="changePanelType"
      @panel-maximized-full="setMaximizedFull($event)"
      @panel-maximized-container="setMaximizedContainer($event)"
      @panel-minimized-full="setMaximizedFull($event)"
      @panel-minimized-container="setMaximizedContainer($event)"
      :initialMaximizedFull="maximizedFull"
      :initialMaximizedContainer="maximizedContainer"
      :isMaximizedContainerEnabled="isMaximizedContainerEnabled"
      v-else-if="panelType == 'action'"
    />
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import EmptyPanel from "@/organisims/EmptyPanel.vue";
import SecretPanel from "@/organisims/SecretPanel.vue";
import AttributePanel from "@/organisims/AttributePanel.vue";
import SchematicPanel from "@/organisims/SchematicPanel.vue";
import WorkflowPanel from "@/organisims/WorkflowPanel.vue";
import ActionPanel from "@/organisims/ActionPanel.vue";
import { PanelEventBus } from "@/atoms/PanelEventBus";
import Bottle from "bottlejs";
import { Persister } from "@/api/persister";

export enum PanelType {
  Action = "action",
  Attribute = "attribute",
  Secret = "secret",
  Schematic = "schematic",
  Workflow = "workflow",
}

export interface IData {
  shortcuts: boolean;
  panelType: PanelType;
  maximizedFull: boolean;
  maximizedContainer: boolean;
  isVisible: boolean;
  isMaximizedContainerEnabled: Boolean;
}

export default Vue.extend({
  name: "PanelSelector",
  props: {
    panelIndex: Number,
    panelRef: String,
    panelContainerRef: String,
    initialPanelType: {
      type: String as PropType<PanelType>,
      default: PanelType.Schematic,
    },
  },
  components: {
    SecretPanel,
    AttributePanel,
    SchematicPanel,
    WorkflowPanel,
    ActionPanel,
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
        isVisible: true,
        isMaximizedContainerEnabled: true,
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
    hide() {
      this.isVisible = false;
    },
    unhide() {
      this.isVisible = true;
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
