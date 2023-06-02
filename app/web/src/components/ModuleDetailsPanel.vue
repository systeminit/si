<template>
  <RequestStatusMessage
    v-if="getPackageReqStatus.isPending"
    :request-status="getPackageReqStatus"
    show-loader-without-message
  />
  <div v-else-if="selectedModule" class="flex flex-col">
    <div
      class="p-sm border-b dark:border-neutral-600 flex flex-row items-center justify-between"
    >
      <div class="font-bold truncate leading-relaxed">
        {{ selectedModule.name }}
      </div>
      <VButton
        :disabled="disableInstallButton"
        :loading="disableInstallButton"
        :label="selectedModule.installed ? 'Remove' : 'Install'"
        :loading-text="
          selectedModule.installed ? 'Removing...' : 'Installing...'
        "
        tone="action"
        icon="plus"
        size="md"
        @click="installModule"
      />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";

import { VButton, RequestStatusMessage } from "@si/vue-lib/design-system";
import { useModuleStore } from "../store/module.store";

const moduleStore = useModuleStore();
const getPackageReqStatus = moduleStore.getRequestStatus("GET_MODULE");
const disableInstallButton = ref(false);

const selectedModule = computed(() => moduleStore.selectedPackage);
const selectedModuleListItem = computed(
  () => moduleStore.selectedPackageListItem,
);

const installModule = async () => {
  disableInstallButton.value = true;
  if (selectedModuleListItem.value) {
    await moduleStore.INSTALL_MODULE(selectedModuleListItem.value);
  }
  disableInstallButton.value = false;
};
</script>
