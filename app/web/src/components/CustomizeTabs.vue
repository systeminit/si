<template>
  <div class="relative flex-grow">
    <SiTabGroup :selected-index="selectedIndex" :on-change="switchTab">
      <template #tabs>
        <SiTabHeader :key="0">FUNCTIONS</SiTabHeader>
        <SiTabHeader :key="1">PACKAGES</SiTabHeader>
        <SiTabHeader :key="2">ASSETS</SiTabHeader>
      </template>
      <template #panels>
        <TabPanel v-for="index in selectedIndex" :key="index" />
        <TabPanel :key="2" class="h-full overflow-auto flex flex-col">
          <slot />
        </TabPanel>
      </template>
      <slot name="modal" />
    </SiTabGroup>
  </div>
</template>

<script lang="ts" setup>
import { TabPanel } from "@headlessui/vue";
import { useRouter } from "vue-router";
import SiTabHeader from "./SiTabHeader.vue";
import SiTabGroup from "./SiTabGroup.vue";

const router = useRouter();

defineProps({
  selectedIndex: { type: Number, required: true },
});

const switchTab = (index: number) => {
  if (index === 0) {
    router.push({ name: "workspace-lab-functions" });
  } else if (index === 1) {
    router.push({ name: "workspace-lab-packages" });
  } else if (index === 2) {
    router.push({ name: "workspace-lab-assets" });
  }
};
</script>
