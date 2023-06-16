<template>
  <WorkspaceSinglePageNew
    v-if="featureFlagsStore.SINGLE_MODEL_SCREEN"
    :workspace-pk="props.workspacePk"
    :change-set-id="props.changeSetId"
  />
  <WorkspaceSinglePageOld
    v-else-if="featureFlagsStore.SINGLE_MODEL_SCREEN === false"
    :workspace-pk="props.workspacePk"
    :change-set-id="props.changeSetId"
  />
</template>

<script lang="ts" setup>
import WorkspaceSinglePageNew from "@/pages/WorkspaceSinglePageNew.vue";
import WorkspaceSinglePageOld from "@/pages/WorkspaceSinglePageOld.vue";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";

const props = defineProps<{
  workspacePk: string;
  changeSetId: string | "auto";
}>();

const featureFlagsStore = useFeatureFlagsStore();
</script>
