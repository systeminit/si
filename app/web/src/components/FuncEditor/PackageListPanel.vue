<template>
  <div>
    <RequestStatusMessage
      :request-status="loadPackagesReqStatus"
      loading-message="Loading packages..."
    />
    <template v-if="loadPackagesReqStatus.isSuccess">
      <div
        class="w-full p-2 border-b dark:border-neutral-600 flex gap-1 flex-row-reverse"
      >
        <!-- TODO - currently this button doesn't do anything -->
        <VButton2 label="Package" tone="action" icon="plus" size="sm" />
      </div>
      <SiSearch auto-search placeholder="search packages" />
      <div
        class="w-full text-neutral-400 dark:text-neutral-300 text-sm text-center p-2 border-b dark:border-neutral-600"
      >
        Select a package to view or edit it.
      </div>
      <SiCollapsible label="Installed Packages" default-open>
        <ul class="overflow-y-auto">
          <li
            v-if="!packageStore.installedPackages.length"
            class="p-sm italic text-center text-xs"
          >
            No packages installed.
          </li>
          <li v-for="p in packageStore.installedPackages" :key="p.id">
            <SiPackageListItem :package-id="p.id" />
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
          <li v-for="p in packageStore.notInstalledPackages" :key="p.id">
            <SiPackageListItem :package-id="p.id" />
          </li>
        </ul>
      </SiCollapsible>
    </template>
  </div>
</template>

<script lang="ts" setup>
import SiPackageListItem from "@/components/SiPackageListItem.vue";
import SiSearch from "@/components/SiSearch.vue";
import { usePackageStore } from "@/store/package.store";
import VButton2 from "@/ui-lib/VButton2.vue";
import RequestStatusMessage from "@/ui-lib/RequestStatusMessage.vue";
import SiCollapsible from "../SiCollapsible.vue";

const packageStore = usePackageStore();
const loadPackagesReqStatus = packageStore.getRequestStatus("LOAD_PACKAGES");
</script>
