<template>
  <SiPanel remember-size-key="workflow-left" side="left" :min-size="220">
    <FixPicker />
  </SiPanel>
  <div class="grow h-full relative bg-neutral-50 dark:bg-neutral-900">
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
import _ from "lodash";
import SiPanel from "@/atoms/SiPanel.vue";

import { useComponentsStore } from "@/store/components.store";
import FixPicker from "../FixPicker.vue";
import FixHistory from "../FixHistory.vue";
import GenericDiagram from "../GenericDiagram/GenericDiagram.vue";

const componentsStore = useComponentsStore();
const diagramNodes = computed(() => componentsStore.diagramNodes);
const diagramEdges = computed(() => componentsStore.diagramEdges);
</script>
