<template>
  <SiTabGroup>
    <template #tabs>
      <SiTabHeader>Diagram Outline</SiTabHeader>
      <SiTabHeader>Local Assets</SiTabHeader>
    </template>

    <template #panels>
      <TabPanel class="flex flex-col overflow-y-hidden">
        <SchematicOutline :viewer-event$="props.viewerEvent$" />
      </TabPanel>
      <TabPanel>Local Assets</TabPanel>
    </template>
  </SiTabGroup>
</template>

<script lang="ts" setup>
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import { TabPanel } from "@headlessui/vue";
import { ViewerEventObservable } from "./SiCanvas/viewer_event";
import SchematicOutline from "@/organisms/SchematicOutline.vue";

const props = defineProps<{
  viewerEvent$: ViewerEventObservable["viewerEvent$"];
}>();
</script>
