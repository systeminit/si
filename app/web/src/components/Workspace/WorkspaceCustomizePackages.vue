<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <SiPanel remember-size-key="func-picker" side="left" :min-size="300">
    <div class="flex flex-col h-full">
      <ChangeSetPanel
        class="border-b-2 dark:border-neutral-500 mb-2 flex-shrink-0"
      />
      <CustomizeTabs :selected-index="1">
        <PackageListPanel :slug="packageSlug" />
      </CustomizeTabs>
    </div>
  </SiPanel>
  <div
    class="grow overflow-hidden bg-shade-0 dark:bg-neutral-800 dark:text-shade-0 text-lg font-semi-bold flex flex-col relative"
  >
    <div class="inset-2 bottom-0 absolute w-full h-full">
      <PackageDisplay :slug="packageSlug" />
    </div>
  </div>
  <SiPanel remember-size-key="func-details" side="right" :min-size="200">
    <PackageDetailsPanel :slug="packageSlug" />
  </SiPanel>
</template>

<script lang="ts" setup>
import { watch } from "vue";
import _ from "lodash";
import ChangeSetPanel from "@/components/ChangeSetPanel.vue";
import PackageListPanel from "@/components/FuncEditor/PackageListPanel.vue";
import PackageDisplay from "@/components/PackageDisplay.vue";
import PackageDetailsPanel from "@/components/PackageDetailsPanel.vue";
import { usePackageStore } from "@/store/package.store";
import SiPanel from "@/components/SiPanel.vue";
import CustomizeTabs from "../CustomizeTabs.vue";

const packageStore = usePackageStore();
const loadPackagesReqStatus = packageStore.getRequestStatus("LOAD_PACKAGES");

const props = defineProps<{
  packageSlug?: string;
  workspacePk: string;
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
