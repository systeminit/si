<template>
  <WorkspaceSinglePageNew
    v-if="featureFlagsStore.SINGLE_MODEL_SCREEN"
    :workspacePk="props.workspacePk"
    :changeSetId="props.changeSetId"
  />
  <WorkspaceSinglePageOld
    v-else
    :workspacePk="props.workspacePk"
    :changeSetId="props.changeSetId"
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
