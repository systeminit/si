<template>
  <div id="panelTreeRoot" class="flex flex-col w-full h-full">
    <PanelContainer
      v-for="(panelContainer, panelContainerIndex) in panelContainers"
      :key="panelContainerIndex"
      :maximized-full-panel="maximizedData"
      :panel-container="panelContainer"
      parent-prefix="root"
      :index="panelContainerIndex"
      @panel-maximize-full="maximizePanelFull($event)"
      @panel-minimize-full="minimizePanelFull($event)"
    />
  </div>
</template>

<script setup lang="ts">
import type { IPanelContainer, PanelMaximized } from "./PanelTree/panel_types";
import { PanelType } from "./PanelTree/panel_types";
import PanelContainer from "./PanelTree/PanelContainer.vue";
import { ref, watch, onBeforeMount } from "vue";

const maximizedData = ref<PanelMaximized | null>(null);

watch(
  maximizedData,
  (maximizedData) => {
    sessionStorage.setItem(
      "panelTreeRootMaximized",
      JSON.stringify(maximizedData),
    );
  },
  { deep: true },
);

onBeforeMount(() => {
  let item = sessionStorage.getItem("panelTreeRootMaximized");
  if (item) {
    maximizedData.value = JSON.parse(item);
  }
});

const panelContainers = ref<IPanelContainer[]>([
  {
    orientation: "row",
    type: "panelContainer",
    panels: [
      {
        orientation: "column",
        type: "panelContainer",
        width: 60,
        panels: [
          {
            name: PanelType.Schematic,
            type: "panel",
          },
          {
            name: PanelType.Schematic,
            type: "panel",
          },
        ],
      },
      {
        orientation: "column",
        type: "panelContainer",
        panels: [
          {
            name: PanelType.Attribute,
            type: "panel",
          },
          {
            name: PanelType.Attribute,
            type: "panel",
          },
        ],
      },
    ],
  },
]);

const minimizePanelFull = (_event: PanelMaximized) => {
  maximizedData.value = null;
};

const maximizePanelFull = (event: PanelMaximized) => {
  maximizedData.value = event;
};
</script>
