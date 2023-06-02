<template>
  <WorkspaceModelAndViewNew v-if="featureFlagsStore.SINGLE_MODEL_SCREEN" />
  <WorkspaceModelAndViewOld v-else />
</template>

<script lang="ts" setup>
import WorkspaceModelAndViewNew from "@/components/Workspace/WorkspaceModelAndViewNew.vue";
import WorkspaceModelAndViewOld from "@/components/Workspace/WorkspaceModelAndViewOld.vue";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";

const featureFlagsStore = useFeatureFlagsStore();
</script>
