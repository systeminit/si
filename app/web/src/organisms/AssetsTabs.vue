<template>
  <SiTabGroup :start-margin="4">
    <template #tabs>
      <SiTabHeader after-margin="2">Asset Palette</SiTabHeader>
      <SiTabHeader after-margin="2">Local Assets</SiTabHeader>
      <SiTabHeader after-margin="2">Panel</SiTabHeader>
    </template>

    <template #panels>
      <TabPanel class="flex flex-col overflow-y-hidden">
        <AssetPalette :viewer-event$="props.viewerEvent$" />
      </TabPanel>
      <TabPanel>Local Assets</TabPanel>
      <TabPanel>Panel</TabPanel>
    </template>
  </SiTabGroup>
</template>

<script setup lang="ts">
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import { TabPanel } from "@headlessui/vue";
import AssetPalette from "@/organisms/AssetPalette.vue";
import { ViewerEventObservable } from "./SiCanvas/viewer_event";

const props = defineProps<{
  viewerEvent$: ViewerEventObservable["viewerEvent$"];
}>();
</script>
