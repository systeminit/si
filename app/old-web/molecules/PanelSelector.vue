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
      :initialContext="attributePanelInitialContext"
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
      :initialContext="schematicPanelInitialContext"
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
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import SecretPanel from "@/organisims/SecretPanel.vue";
import AttributePanel from "@/organisims/AttributePanel.vue";
import SchematicPanel from "@/organisims/SchematicPanel.vue";

import { schematicKindfromString } from "@/api/sdf/model/schematic";

import { PanelEventBus } from "@/atoms/PanelEventBus";
import { panelTypeChanges$, restorePanelTypeChanges$ } from "@/observables";
import { tap } from "rxjs/operators";

export enum PanelType {
  Attribute = "attribute",
  Secret = "secret",
  Schematic = "schematic",
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
    initialContextType: {
      type: String,
    },
  },
  components: {
    SecretPanel,
    AttributePanel,
    SchematicPanel,
  },
  data(): IData {
    return {
      shortcuts: false,
      panelType: this.initialPanelType,
      maximizedFull: false,
      maximizedContainer: false,
      isVisible: true,
      isMaximizedContainerEnabled: true,
    };
  },
  mounted() {
    PanelEventBus.$on("shortcut", this.handleShortcut);
  },
  beforeDestroy() {
    PanelEventBus.$off("shortcut", this.handleShortcut);
  },
  subscriptions(): Record<string, any> {
    return {
      restorePanelType: restorePanelTypeChanges$.pipe(
        tap(panelTypeChange => {
          if (panelTypeChange) {
            // TODO: ew. we shouldn't couple to the router, but.. it's fine for now
            let applicationId = this.$route.params["applicationId"];
            if (
              // @ts-ignore
              this.panelRef == panelTypeChange.panelRef &&
              applicationId == panelTypeChange.applicationId
            ) {
              // @ts-ignore
              this.changePanelType(panelTypeChange.panelType);
            }
          }
        }),
      ),
    };
  },
  computed: {
    schematicPanelInitialContext() {
      return schematicKindfromString(this.initialContextType);
    },
    attributePanelInitialContext() {
      return this.initialContextType;
    },
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
        // keyCode 219 is "["
        if (
          (event.altKey && event.shiftKey && event.key == "N") ||
          event.keyCode == 219
        ) {
          PanelEventBus.$emit("create-new-panel", {
            panelContainerRef: this.panelContainerRef,
          });
        }

        // keyCode 221 is "]"
        if (
          (event.altKey && event.shiftKey && event.key == "D") ||
          event.keyCode == 221
        ) {
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
      // TODO: ew. we shouldn't couple to the router, but.. it's fine for now
      let applicationId = this.$route.params["applicationId"];
      panelTypeChanges$.next({
        panelRef: this.panelRef,
        // @ts-ignore
        applicationId,
        panelType: newPanelType,
      });
    },
  },
});
</script>
