<template>
  <SiTabGroup>
    <template #tabs>
      <SiTabHeader>Attributes</SiTabHeader>
      <SiTabHeader>Code</SiTabHeader>
    </template>

    <template #panels>
      <TabPanel class="flex flex-col overflow-y-hidden">
        <div class="text-center mt-10">
          Selected node: {{ activeNode?.name ?? "" }}
        </div>
      </TabPanel>
      <TabPanel>
        <div>Code Content for {{ activeNode?.name }}</div>
      </TabPanel>
    </template>
  </SiTabGroup>
</template>

<script setup lang="ts">
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import { TabPanel } from "@headlessui/vue";
import { refFrom } from "vuse-rx";
import { lastSelectedNode$ } from "@/observable/selection";

const activeNode = refFrom(lastSelectedNode$);
</script>
