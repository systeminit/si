<template>
  <ScrollArea>
    <RequestStatusMessage
      :request-status="loadPackagesReqStatus"
      loading-message="Loading modules..."
    />
    <template v-if="loadPackagesReqStatus.isSuccess" #top>
      <div
        class="w-full p-2 border-b dark:border-neutral-600 flex gap-1 flex-row-reverse"
      >
        <VButton
          label="Module"
          tone="action"
          icon="plus"
          size="sm"
          @click="openModal"
        />
      </div>
      <SiSearch auto-search placeholder="search modules" />
      <div
        class="w-full text-neutral-400 dark:text-neutral-300 text-sm text-center p-2 border-b dark:border-neutral-600"
      >
        Select a module to view or edit it.
      </div>
    </template>
    <template v-if="loadPackagesReqStatus.isSuccess">
      <SiCollapsible label="Installed Modules" default-open>
        <ul class="overflow-y-auto">
          <li
            v-if="!moduleStore.installedPackages.length"
            class="p-sm italic text-center text-xs"
          >
            No modules installed.
          </li>
          <li v-for="p in moduleStore.installedPackages" :key="p.name">
            <SiPackageListItem :package-id="p.name" />
          </li>
        </ul>
      </SiCollapsible>
      <SiCollapsible label="Available Modules" default-open>
        <ul class="overflow-y-auto">
          <li
            v-if="!moduleStore.notInstalledPackages.length"
            class="p-sm italic text-center text-xs"
          >
            All available modules are already installed.
          </li>
          <li v-for="p in moduleStore.notInstalledPackages" :key="p.name">
            <SiPackageListItem :package-id="p.name" />
          </li>
        </ul>
      </SiCollapsible>
      <ModuleExportModal ref="exportModalRef" />
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
import { useModuleStore } from "../store/module.store";
import SiCollapsible from "./SiCollapsible.vue";
import ModuleExportModal from "./ModuleExportModal.vue";

const moduleStore = useModuleStore();
const loadPackagesReqStatus = moduleStore.getRequestStatus("LOAD_MODULES");
const exportModalRef = ref<InstanceType<typeof Modal>>();

const openModal = () => {
  exportModalRef.value?.open();
};
</script>
