<template>
  <div v-if="packageStore.selectedPackage" class="flex flex-col">
    <div
      class="p-sm border-b dark:border-neutral-600 flex flex-row items-center justify-between"
    >
      <div class="font-bold truncate leading-relaxed">
        {{ packageStore.selectedPackage.displayName }}
      </div>
      <VButton2 label="Import" tone="action" icon="plus" size="md" />
    </div>
    <div class="p-sm flex flex-col">
      <div class="pb-xs font-bold text-xl">Changelog:</div>
      <div>{{ packageStore.selectedPackage.changelog }}</div>
    </div>
  </div>
  <div
    v-else
    class="px-2 py-sm text-center text-neutral-400 dark:text-neutral-300"
  >
    <template v-if="slug">Package "{{ slug }}" does not exist!</template>
    <template v-else>Select a package to view its changelog.</template>
  </div>
</template>

<script lang="ts" setup>
import VButton2 from "@/ui-lib/VButton2.vue";

import { usePackageStore } from "@/store/package.store";

const packageStore = usePackageStore();

defineProps<{
  slug?: string;
}>();
</script>
