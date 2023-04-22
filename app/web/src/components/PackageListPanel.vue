<template>
  <ScrollArea>
    <RequestStatusMessage
      :request-status="loadPackagesReqStatus"
      loading-message="Loading packages..."
    />
    <template v-if="loadPackagesReqStatus.isSuccess" #top>
      <div
        class="w-full p-2 border-b dark:border-neutral-600 flex gap-1 flex-row-reverse"
      >
        <VButton
          label="Package"
          tone="action"
          icon="plus"
          size="sm"
          @click="openModal"
        />
      </div>
      <SiSearch auto-search placeholder="search packages" />
      <div
        class="w-full text-neutral-400 dark:text-neutral-300 text-sm text-center p-2 border-b dark:border-neutral-600"
      >
        Select a package to view or edit it.
      </div>
    </template>
    <template v-if="loadPackagesReqStatus.isSuccess">
      <SiCollapsible label="Installed Packages" default-open>
        <ul class="overflow-y-auto">
          <li
            v-if="!packageStore.installedPackages.length"
            class="p-sm italic text-center text-xs"
          >
            No packages installed.
          </li>
          <li v-for="p in packageStore.installedPackages" :key="p.name">
            <SiPackageListItem :package-id="p.name" />
          </li>
        </ul>
      </SiCollapsible>
      <SiCollapsible label="Available Packages" default-open>
        <ul class="overflow-y-auto">
          <li
            v-if="!packageStore.notInstalledPackages.length"
            class="p-sm italic text-center text-xs"
          >
            All available packages are already installed.
          </li>
          <li v-for="p in packageStore.notInstalledPackages" :key="p.name">
            <SiPackageListItem :package-id="p.name" />
          </li>
        </ul>
      </SiCollapsible>
      <PackageExportModal ref="exportModalRef" />
    </template>
  </ScrollArea>
</template>

<script lang="ts" setup>
import { ref } from "vue";
import {
  Modal,
  RequestStatusMessage,
  ScrollArea,
  VButton,
} from "@si/vue-lib/design-system";
import SiPackageListItem from "@/components/SiPackageListItem.vue";
import SiSearch from "@/components/SiSearch.vue";
import { usePackageStore } from "@/store/package.store";
import SiCollapsible from "./SiCollapsible.vue";
import PackageExportModal from "./PackageExportModal.vue";

const packageStore = usePackageStore();
const loadPackagesReqStatus = packageStore.getRequestStatus("LOAD_PACKAGES");
const exportModalRef = ref<InstanceType<typeof Modal>>();

const openModal = () => {
  exportModalRef.value?.open();
};
</script>
