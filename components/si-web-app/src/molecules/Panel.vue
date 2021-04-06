<template>
  <div
    class="flex flex-col w-full h-ful"
    :class="panelClasses()"
    @mouseenter="mouseEnter()"
    @mouseleave="mouseLeave()"
    v-if="isVisible"
  >
    <div
      class="flex flex-row items-center w-full bg-black"
      :class="panelMenuClasses()"
      style="height: 2.5rem; min-height: 2.5rem"
    >
      <div class="flex justify-start">
        <SiSelect
          size="xs"
          :options="panelTypes"
          id="selectPanelType"
          v-model="selectedPanelType"
          class="pl-2"
          @change.native="changePanelType"
        />
      </div>
      <div class="flex justify-start">
        <slot name="menuButtons"> </slot>
      </div>
      <div class="flex flex-row items-center justify-end flex-grow">
        <div class="flex items-center h-full pr-2">
          <button
            data-testid="minimize-container"
            @click="minimizeContainer"
            v-if="maximizedContainer && !maximizedFull"
          >
            <Minimize2Icon size="1.2x" />
          </button>

          <button
            data-testid="maximize-container"
            @click="maximizeContainer"
            v-if="
              !maximizedContainer &&
                !maximizedFull &&
                isMaximizedContainerEnabled
            "
          >
            <Maximize2Icon size="1.2x" />
          </button>
        </div>
        <div class="flex items-center h-full pr-2">
          <button
            data-testid="minimize-full"
            @click="minimizeFull"
            v-if="maximizedFull"
          >
            <MinimizeIcon size="1.2x" />
          </button>

          <button data-testid="maximize-full" @click="maximizeFull" v-else>
            <MaximizeIcon size="1.2x" />
          </button>
        </div>
      </div>
    </div>
    <div class="flex flex-row w-full h-full overflow-auto bg-gray-900">
      <slot name="content"> </slot>
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import SiSelect from "@/atoms/SiSelect.vue";
import { ILabelList } from "@/api/sdf/dal";
import {
  Minimize2Icon,
  Maximize2Icon,
  MinimizeIcon,
  MaximizeIcon,
} from "vue-feather-icons";
import { PanelType } from "@/molecules/PanelSelector.vue";
import { PanelEventBus } from "@/atoms/PanelEventBus";
import _ from "lodash";

import {
  ShortcutRegistrationEvent,
  ShortcutContext,
  MouseRegistrationEvent,
  MouseContext,
  ShortcutActions,
  ShortcutUpdateEvent,
} from "@/organisims/ShortcutsEventBroker.vue";

interface IData {
  selectedPanelType: PanelType;
  maximizedFull: boolean;
  maximizedContainer: boolean;
  shortcutsEnabled: boolean;
  id: string;
  isActive: boolean;
}

export default Vue.extend({
  name: "Panel",
  props: {
    panelIndex: Number,
    panelRef: String,
    panelContainerRef: String,
    initialPanelType: String as PropType<PanelType>,
    initialMaximizedFull: Boolean,
    initialMaximizedContainer: Boolean,
    isVisible: {
      type: Boolean,
      default: true,
    },
    isMaximizedContainerEnabled: Boolean,
  },
  components: {
    SiSelect,
    Maximize2Icon,
    Minimize2Icon,
    MinimizeIcon,
    MaximizeIcon,
  },
  data() {
    return {
      selectedPanelType: this.initialPanelType,
      maximizedFull: this.initialMaximizedFull,
      maximizedContainer: this.initialMaximizedContainer,
      shortcutsEnabled: false,
      id: _.uniqueId("panel-"),
      isActive: false,
    };
  },
  computed: {
    panelSelectId(): string {
      return `${this.panelRef}-selectPanelType`;
    },
    panelTypes(): ILabelList {
      return [
        {
          label: "Schematic",
          value: PanelType.Schematic,
        },
        {
          label: "Attribute",
          value: PanelType.Attribute,
        },
        {
          label: "Secret",
          value: PanelType.Secret,
        },
      ];
    },
  },
  mounted: function() {
    this.registerEvents();
  },
  beforeDestroy() {
    this.deRegisterEvents();
  },
  updated() {
    // Ideally this should only update the child instance...
    PanelEventBus.$emit("panel-viewport-update", true);
  },
  methods: {
    registerEvents(): void {
      PanelEventBus.$on(
        "shortcuts-update-" + this.id,
        this.handleShortcutUpdate,
      );
    },
    deRegisterEvents(): void {
      PanelEventBus.$off(
        "shortcuts-update-" + this.id,
        this.handleShortcutUpdate,
      );
    },
    handleShortcutUpdate(e: ShortcutUpdateEvent) {
      if ((e.action = ShortcutActions["Maximize"])) {
        if (!this.maximizedFull) {
          this.maximizeFull();
        } else {
          this.minimizeFull();
        }
      }
    },
    changePanelType() {
      this.$emit("change-panel", this.selectedPanelType);
    },
    mouseEnter() {
      this.isActive = true;
      this.activateShortcuts();
    },
    mouseLeave() {
      this.isActive = false;
      this.deactivateShortcuts();
    },
    activateShortcuts() {
      this.shortcutsEnabled = true;

      let ctx: ShortcutContext = {
        id: this.id,
        isActive: true,
      };
      let event: ShortcutRegistrationEvent = {
        context: ctx,
      };
      PanelEventBus.$emit("shortcuts-registration-update", event);
    },
    deactivateShortcuts() {
      this.shortcutsEnabled = false;

      let ctx: ShortcutContext = {
        id: this.id,
        isActive: false,
      };
      let event: ShortcutRegistrationEvent = {
        context: ctx,
      };

      PanelEventBus.$emit("shortcuts-registration-update", event);
    },
    maximizeContainer() {
      this.maximizedContainer = true;
      this.$emit("panel-maximized-container", true);
      PanelEventBus.$emit("maximize-container", {
        panelIndex: this.panelIndex,
        panelRef: this.panelRef,
        panelContainerRef: this.panelContainerRef,
      });
    },
    minimizeContainer() {
      this.maximizedContainer = false;
      this.$emit("panel-maximized-container", false);
      PanelEventBus.$emit("minimize-container", {
        panelIndex: this.panelIndex,
        panelRef: this.panelRef,
        panelContainerRef: this.panelContainerRef,
      });
    },
    maximizeFull() {
      this.maximizedFull = true;
      this.$emit("panel-maximized-full", true);
      PanelEventBus.$emit("maximize-full", {
        panelRef: this.panelRef,
        panelContainerRef: this.panelContainerRef,
      });
    },
    minimizeFull() {
      this.maximizedFull = false;
      this.$emit("panel-minimized-full", false);
      PanelEventBus.$emit("minimize-full", {
        panelRef: this.panelRef,
        panelContainerRef: this.panelContainerRef,
      });
    },
    panelClasses(): Record<string, any> {
      let classes: Record<string, any> = {};
      // classes["hidden"] = !this.isVisible;
      // classes["overflow-hidden"] = !this.isVisible;
      // classes["active-panel"] = this.isActive;
      classes["inactive-panel"] = !this.isActive;
      return classes;
    },
    panelMenuClasses(): Record<string, any> {
      let classes: Record<string, any> = {};
      classes["inactive-panel-menu"] = !this.isActive;
      return classes;
    },
  },
});
</script>

<style scoped>
div.inactive-panel-menu > * {
  filter: brightness(90%);
}

div.active-panel {
  /* border: solid;
  border-color: #323536;
  border-width: 0.1em; */
}

div.inactive-panel > * {
  /* filter: brightness(98%); */
}
</style>
