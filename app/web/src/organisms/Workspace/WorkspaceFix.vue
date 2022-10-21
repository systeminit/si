<template>
  <SiPanel remember-size-key="workflow-left" side="left" :min-size="315">
    <FixPicker />
  </SiPanel>
  <div class="grow h-full relative bg-neutral-50 dark:bg-neutral-900">
    <FixProgressOverlay />
    <GenericDiagram
      v-if="diagramNodes"
      :custom-config="diagramCustomConfig"
      :nodes="diagramNodes"
      :edges="diagramEdges"
      read-only
    />
  </div>
  <SiPanel remember-size-key="workflow-right" side="right" :min-size="280">
    <FixHistory />
  </SiPanel>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import SiPanel from "@/atoms/SiPanel.vue";
import { useComponentsStore } from "@/store/components.store";
import FixProgressOverlay from "@/organisms/FixProgressOverlay.vue";
import FixPicker from "../FixPicker.vue";
import FixHistory from "../FixHistory.vue";
import GenericDiagram from "../GenericDiagram/GenericDiagram.vue";

const componentsStore = useComponentsStore();
const diagramNodes = computed(() => componentsStore.diagramNodes);
const diagramEdges = computed(() => componentsStore.diagramEdges);

const diagramCustomConfig = {};
</script>
