<template>
  <SiTabGroup :selected-index="1" @change="onTabChange">
    <template #tabs>
      <SiTabHeader :key="0">FUNCTIONS</SiTabHeader>
      <SiTabHeader :key="1">PACKAGES</SiTabHeader>
    </template>
    <template #panels>
      <TabPanel />
      <TabPanel :key="1" class="h-full overflow-auto flex flex-col">
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
              <li
                v-for="(p, index) in packageStore.installedPackages"
                :key="index"
              >
                <SiPackageListItem :p="p" />
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
              <li
                v-for="(p, index) in packageStore.notInstalledPackages"
                :key="index"
              >
                <SiPackageListItem :p="p" />
              </li>
            </ul>
          </SiCollapsible>
        </template>
      </TabPanel>
    </template>
  </SiTabGroup>
</template>

<script lang="ts" setup>
import { TabPanel } from "@headlessui/vue";
import _ from "lodash";
import { useRouter } from "vue-router";
import SiPackageListItem from "@/molecules/SiPackageListItem.vue";
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import SiSearch from "@/molecules/SiSearch.vue";
import { usePackageStore } from "@/store/package.store";
import VButton2 from "@/ui-lib/VButton2.vue";
import RequestStatusMessage from "@/ui-lib/RequestStatusMessage.vue";
import SiCollapsible from "../SiCollapsible.vue";

const router = useRouter();
const packageStore = usePackageStore();
const loadPackagesReqStatus = packageStore.getRequestStatus("LOAD_PACKAGES");

const onTabChange = () => {
  router.push({ name: "workspace-lab-functions" });
};
</script>
