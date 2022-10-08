<template>
  <SiPanel remember-size-key="workflow-left" side="left" :min-size="315">
    <FixPicker />
  </SiPanel>
  <div class="grow h-full relative bg-neutral-50 dark:bg-neutral-900">
    <FixProgressOverlay />
    <GenericDiagram
      v-if="diagramNodes"
      :nodes="diagramNodes"
      :edges="diagramEdges"
      read-only
    />
  </div>
  <SiPanel remember-size-key="workflow-right" side="right">
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

/*
 * - Analyze fixes from diagram/show status bar/spinner every component (if we can!)/jitter/Populate fix list
 * - wait for fix resources click:
 * - Spin every item of fix list/sync with "Applying Fixes"/Fixed items go to "Fixed list"
 * */
</script>
