<template>
  <SiTabGroup>
    <template #tabs>
      <SiTabHeader>Asset Palette</SiTabHeader>
      <SiTabHeader>Local Assets</SiTabHeader>
      <SiTabHeader>Panel</SiTabHeader>
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
import { ViewerEventObservable } from "./SchematicViewer/viewer_event";

const props = defineProps<{
  viewerEvent$: ViewerEventObservable["viewerEvent$"];
}>();
</script>
