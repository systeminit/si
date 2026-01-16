<template>
  <RequestStatusMessage v-if="localDetailsReq.isPending" :requestStatus="localDetailsReq" />
  <div v-else-if="localDetails" class="flex flex-col">
    <div class="p-sm border-b dark:border-neutral-600 flex flex-row items-center justify-between">
      <div class="font-bold truncate leading-relaxed">
        {{ localDetails.name }}
      </div>

      <!-- <VButton
        v-if="localDetails"
        label="Uninstall"
        loading-text="Removing..."
        tone="destructive"
        icon="trash"
        size="md"
        @click="uninstallModule"
      /> -->
      <!-- <VButton
        v-if="!localDetails"
        label="Install"
        loading-text="Installing"
        tone="action"
        icon="plus"
        size="md"
        @click="installModule"
      /> -->
    </div>
  </div>
  <EmptyStateCard
    v-else
    iconName="no-changes"
    primaryText="No Module Selected"
    secondaryText="Select a module from the list on the left panel to view its details here."
  />
</template>

<script lang="ts" setup>
import { computed } from "vue";
import { RequestStatusMessage } from "@si/vue-lib/design-system";
import { useModuleStore } from "@/store/module.store";
import EmptyStateCard from "../EmptyStateCard.vue";

const moduleStore = useModuleStore();
const localDetailsReq = moduleStore.getRequestStatus("GET_LOCAL_MODULE_DETAILS");
// const remoteDetailsReq = moduleStore.getRequestStatus(
//   "GET_REMOTE_MODULE_DETAILS",
// );

const localDetails = computed(() => moduleStore.selectedModuleLocalDetails);
// const remoteDetails = computed(() => moduleStore.selectedModuleRemoteDetails);

// async function installModule() {
//   if (!remoteDetails.value) return;
//   await moduleStore.INSTALL_REMOTE_MODULE(remoteDetails.value?.id);
// }
// async function uninstallModule() {
//   alert("Uninstall not yet supported");
// }
</script>
