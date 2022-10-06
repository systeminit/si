<template>
  <SiPanel remember-size-key="workflow-left" side="left" :min-size="315">
    <FixPicker />
  </SiPanel>
  <div class="grow h-full relative bg-neutral-50 dark:bg-neutral-900">
    <div
      :class="
        clsx(
          'absolute z-20 left-0 right-0 mx-4 mt-3 p-4',
          'bg-white dark:bg-neutral-800 dark:text-white border border-neutral-300 dark:border-neutral-600',
          'shadow-md rounded-md font-bold',
        )
      "
    >
      <div>Determining fixes for updated model...</div>

      <div class="mt-2 flex gap-5">
        <ProgressBar :completion-rate="fixComponentsProgress" />
        <span class="whitespace-nowrap flex-shrink-0">
          {{ fixesStore.processedFixComponents }} of
          {{ fixesStore.totalFixComponents }} Synced
        </span>
      </div>
    </div>
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
import clsx from "clsx";
import SiPanel from "@/atoms/SiPanel.vue";

import { useComponentsStore } from "@/store/components.store";
import ProgressBar from "@/atoms/ProgressBar.vue";
import { useFixesStore } from "@/store/fixes.store";
import FixPicker from "../FixPicker.vue";
import FixHistory from "../FixHistory.vue";
import GenericDiagram from "../GenericDiagram/GenericDiagram.vue";

const componentsStore = useComponentsStore();
const diagramNodes = computed(() => componentsStore.diagramNodes);
const diagramEdges = computed(() => componentsStore.diagramEdges);

const fixesStore = useFixesStore();
const loadFixesReqStatus = fixesStore.getRequestStatus("LOAD_FIXES");

const fixComponentsProgress = computed(() => {
  if (loadFixesReqStatus.value.isSuccess && fixesStore.totalFixComponents > 0) {
    return fixesStore.processedFixComponents / fixesStore.totalFixComponents;
  }
  return 0;
});

/*
 * - Analyze fixes from diagram/show status bar/spinner every component (if we can!)/jitter/Populate fix list
 * - wait for fix resources click:
 * - Spin every item of fix list/sync with "Applying Fixes"/Fixed items go to "Fixed list"
 * */
</script>
