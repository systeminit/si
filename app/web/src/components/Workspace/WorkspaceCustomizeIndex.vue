<template>
  <div
    v-if="FF_SINGLE_MODEL_SCREEN && changeset === null"
    class="relative bg-black w-full h-full flex items-center justify-center text-3xl text-neutral-400"
  >
    Not available on head, create a changeset through the
    <Icon name="git-branch" size="lg" class="inline-block align-middle mx-3" />
    button above to use the customize screen
  </div>
  <RouterView v-else />
</template>

<script lang="ts" setup>
import { Icon } from "@si/vue-lib/design-system";
import { computed } from "vue";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";

const featureFlagsStore = useFeatureFlagsStore();
const FF_SINGLE_MODEL_SCREEN = computed(
  () => featureFlagsStore.SINGLE_MODEL_SCREEN,
);
const changeset = useChangeSetsStore().selectedChangeSet;
</script>
