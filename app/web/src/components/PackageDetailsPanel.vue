<template>
  <div v-if="packageStore.selectedPackage" class="flex flex-col">
    <div
      class="p-sm border-b dark:border-neutral-600 flex flex-row items-center justify-between"
    >
      <div class="font-bold truncate leading-relaxed">
        {{ packageStore.selectedPackage.displayName }}
      </div>
      <VButton2
        :disabled="disableInstallButton"
        :loading="disableInstallButton"
        :label="packageStore.selectedPackage.installed ? 'Remove' : 'Add'"
        :loading-text="
          packageStore.selectedPackage.installed ? 'Removing...' : 'Adding...'
        "
        tone="action"
        icon="plus"
        size="md"
        @click="toggleSelectedPackage()"
      />
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
import { ref } from "vue";
import VButton2 from "@/ui-lib/VButton2.vue";

import { usePackageStore } from "@/store/package.store";

const packageStore = usePackageStore();
const disableInstallButton = ref(false);

defineProps<{
  slug?: string;
}>();

const toggleSelectedPackage = () => {
  disableInstallButton.value = true;
  setTimeout(() => {
    packageStore.selectedPackage.installed =
      !packageStore.selectedPackage.installed;
    disableInstallButton.value = false;
  }, 2000);
};
</script>
