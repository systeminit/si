<template>
  <div class="flex flex-col w-full h-full">
    <div
      class="flex flex-row w-full bg-black items-center"
      style="height: 3rem; min-height: 3rem"
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
      <div class="flex flex-row justify-end flex-grow items-center">
        <div class="pr-2 items-center h-full flex">
          <button
            data-testid="minimize-container"
            @click="minimizeContainer"
            v-if="maximizedContainer"
          >
            <Minimize2Icon />
          </button>
          <button
            data-testid="maximize-container"
            @click="maximizeContainer"
            v-else
          >
            <Maximize2Icon />
          </button>
        </div>
        <div class="pr-2 items-center h-full flex">
          <button
            data-testid="minimize-full"
            @click="minimizeFull"
            v-if="maximizedFull"
          >
            <MinimizeIcon />
          </button>
          <button data-testid="maximize-full" @click="maximizeFull" v-else>
            <MaximizeIcon />
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

interface IData {
  selectedPanelType: PanelType;
  maximizedFull: boolean;
  maximizedContainer: boolean;
}

export default Vue.extend({
  name: "Panel",
  props: {
    panelRef: String,
    panelContainerRef: String,
    initialPanelType: String as PropType<PanelType>,
    initialMaximizedFull: Boolean,
    initialMaximizedContainer: Boolean,
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
    };
  },
  computed: {
    panelSelectId(): string {
      return `${this.panelRef}-selectPanelType`;
    },
    panelTypes(): ILabelList {
      return [
        { label: "", value: PanelType.Empty },
        {
          label: "Schematic",
          value: PanelType.SystemSchematic,
        },
      ];
    },
  },
  methods: {
    changePanelType() {
      this.$emit("change-panel", this.selectedPanelType);
    },
    maximizeContainer() {
      this.maximizedContainer = true;
      this.$emit("panel-maximized-container", true);
      PanelEventBus.$emit("maximize-container", {
        panelRef: this.panelRef,
        panelContainerRef: this.panelContainerRef,
      });
    },
    minimizeContainer() {
      this.maximizedContainer = false;
      this.$emit("panel-maximized-container", false);
      PanelEventBus.$emit("minimize-container", {
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
      this.$emit("panel-maximized-full", false);
      PanelEventBus.$emit("minimize-full", {
        panelRef: this.panelRef,
        panelContainerRef: this.panelContainerRef,
      });
    },
  },
});
</script>
