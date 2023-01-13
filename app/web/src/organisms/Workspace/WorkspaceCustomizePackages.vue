<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <SiPanel remember-size-key="func-picker" side="left" :min-size="300">
    <div class="flex flex-col h-full">
      <ChangeSetPanel
        class="border-b-2 dark:border-neutral-500 mb-2 flex-shrink-0"
      />
      <div class="relative flex-grow">
        <PackageListPanel />
      </div>
    </div>
  </SiPanel>
  <div
    class="grow overflow-hidden bg-shade-0 dark:bg-neutral-800 dark:text-shade-0 text-lg font-semi-bold flex flex-col relative"
  >
    <div class="inset-2 bottom-0 absolute">
      <PackageDisplayPanel />
    </div>
  </div>
  <SiPanel remember-size-key="func-details" side="right" :min-size="200">
    <PackageDetails />
  </SiPanel>
</template>

<script lang="ts" setup>
import { watch } from "vue";
import _ from "lodash";
import SiPanel from "@/atoms/SiPanel.vue";
import ChangeSetPanel from "@/organisms/ChangeSetPanel.vue";
import PackageListPanel from "@/organisms/FuncEditor/PackageListPanel.vue";
import PackageDisplayPanel from "@/organisms/PackageDisplayPanel.vue";
import PackageDetails from "@/organisms/PackageDetails.vue";
import { usePackageStore } from "@/store/package.store";

const packageStore = usePackageStore();
const loadPackagesReqStatus = packageStore.getRequestStatus("LOAD_PACKAGES");

const props = defineProps<{
  packageSlug?: string;
  workspaceId: string;
  changeSetId: string;
}>();

watch(
  [() => props.packageSlug, loadPackagesReqStatus],
  () => {
    if (loadPackagesReqStatus.value.isSuccess && props.packageSlug) {
      packageStore.setSelectedPackageBySlug(props.packageSlug);
    }
  },
  { immediate: true },
);
</script>
