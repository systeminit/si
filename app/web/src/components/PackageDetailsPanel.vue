<template>
  <RequestStatusMessage
    v-if="loadPackagesReqStatus.isPending"
    :request-status="loadPackagesReqStatus"
    show-loader-without-message
  />
  <div v-else-if="selectedPackage" class="flex flex-col">
    <div
      class="p-sm border-b dark:border-neutral-600 flex flex-row items-center justify-between"
    >
      <div class="font-bold truncate leading-relaxed">
        {{ selectedPackage.displayName }}
      </div>
      <VButton2
        :disabled="disableInstallButton"
        :loading="disableInstallButton"
        :label="selectedPackage.installed ? 'Remove' : 'Add'"
        :loading-text="selectedPackage.installed ? 'Removing...' : 'Adding...'"
        tone="action"
        icon="plus"
        size="md"
        @click="toggleInstalled"
      />
    </div>
    <div class="p-sm flex flex-col">
      <div class="pb-xs font-bold text-xl">Changelog:</div>
      <div>{{ selectedPackage.changelog }}</div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import VButton2 from "@/ui-lib/VButton2.vue";

import { usePackageStore } from "@/store/package.store";
import RequestStatusMessage from "@/ui-lib/RequestStatusMessage.vue";

const packageStore = usePackageStore();
const loadPackagesReqStatus = packageStore.getRequestStatus("LOAD_PACKAGES");
const disableInstallButton = ref(false);

const selectedPackage = computed(() => packageStore.selectedPackage);

const toggleInstalled = () => {
  disableInstallButton.value = true;
  setTimeout(() => {
    packageStore.selectedPackage.installed =
      !packageStore.selectedPackage.installed;
    disableInstallButton.value = false;
  }, 2000);
};
</script>
