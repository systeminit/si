<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <SiPanel remember-size-key="workflow-left" side="left" :min-size="315">
    <RecommendationPicker />
  </SiPanel>
  <div class="grow h-full relative bg-neutral-50 dark:bg-neutral-900">
    <RecommendationProgressOverlay />
    <GenericDiagram
      v-if="diagramNodes"
      :custom-config="diagramCustomConfig"
      :nodes="diagramNodes"
      :edges="diagramEdges"
      read-only
    />
  </div>
  <SiPanel remember-size-key="workflow-right" side="right" :min-size="280">
    <ApplyHistory />
  </SiPanel>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import { useComponentsStore } from "@/store/components.store";
import RecommendationProgressOverlay from "@/components/RecommendationProgressOverlay.vue";
import SiPanel from "@/components/SiPanel.vue";
import RecommendationPicker from "../RecommendationPicker.vue";
import ApplyHistory from "../ApplyHistory.vue";
import GenericDiagram from "../GenericDiagram/GenericDiagram.vue";

const componentsStore = useComponentsStore();
const diagramNodes = computed(() => componentsStore.diagramNodes);
const diagramEdges = computed(() => componentsStore.diagramEdges);

const diagramCustomConfig = {};
</script>
