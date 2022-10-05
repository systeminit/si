<template>
  <SiPanel remember-size-key="workflow-left" side="left" :min-size="220">
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
        <ProgressBar :completion-rate="fixData.executed / fixData.total" />
        <span class="whitespace-nowrap flex-shrink-0">
          {{ fixData.executed }} of {{ fixData.total }} Synced
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
import { computed, ref } from "vue";
import clsx from "clsx";
import { interval } from "rxjs";
import { untilUnmounted } from "vuse-rx";
import SiPanel from "@/atoms/SiPanel.vue";

import { useComponentsStore } from "@/store/components.store";
import ProgressBar from "@/atoms/ProgressBar.vue";
import FixPicker from "../FixPicker.vue";
import FixHistory from "../FixHistory.vue";
import GenericDiagram from "../GenericDiagram/GenericDiagram.vue";

const componentsStore = useComponentsStore();
const diagramNodes = computed(() => componentsStore.diagramNodes);
const diagramEdges = computed(() => componentsStore.diagramEdges);

// TODO(victor): This is temporary data just to animate the workspace. It will be removed once we plug the backend in
const fixData = ref({
  executed: 0,
  total: 12,
});

interval(1500)
  .pipe(untilUnmounted)
  .subscribe(() => {
    fixData.value.executed =
      fixData.value.total !== fixData.value.executed
        ? fixData.value.executed + 1
        : 0;
  });
</script>
