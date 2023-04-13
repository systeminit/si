<template>
  <RequestStatusMessage
    v-if="getPackageReqStatus.isPending"
    :request-status="getPackageReqStatus"
    show-loader-without-message
  />
  <div v-else-if="selectedPackage" class="flex flex-col">
    <div
      class="p-sm border-b dark:border-neutral-600 flex flex-row items-center justify-between"
    >
      <div class="font-bold truncate leading-relaxed">
        {{ selectedPackage.name }}
      </div>
      <VButton2
        :disabled="disableInstallButton"
        :loading="disableInstallButton"
        :label="selectedPackage.installed ? 'Remove' : 'Install'"
        :loading-text="
          selectedPackage.installed ? 'Removing...' : 'Installing...'
        "
        tone="action"
        icon="plus"
        size="md"
        @click="installPackage"
      />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";

import { VButton2, RequestStatusMessage } from "@si/vue-lib/design-system";
import { usePackageStore } from "@/store/package.store";

const packageStore = usePackageStore();
const getPackageReqStatus = packageStore.getRequestStatus("GET_PACKAGE");
const disableInstallButton = ref(false);

const selectedPackage = computed(() => packageStore.selectedPackage);
const selectedPackageListItem = computed(
  () => packageStore.selectedPackageListItem,
);

const installPackage = async () => {
  disableInstallButton.value = true;
  if (selectedPackageListItem.value) {
    await packageStore.INSTALL_PACKAGE(selectedPackageListItem.value);
  }
  disableInstallButton.value = false;
};
</script>
