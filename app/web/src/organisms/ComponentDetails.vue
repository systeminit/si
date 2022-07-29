<template>
  <SiTabGroup>
    <template #tabs>
      <SiTabHeader>Attributes</SiTabHeader>
      <SiTabHeader>Code</SiTabHeader>
    </template>

    <template #panels>
      <TabPanel class="flex flex-col overflow-y-auto">
        <div class="text-center">
          <!-- FIXME(nick): remove AttributeViewer's requirement of a componentId -->
          <AttributeViewer
            class="dark:text-neutral-50 text-neutral-900"
            :component-id="props.componentIdentification.componentId"
            :component-identification="props.componentIdentification"
          />
        </div>
      </TabPanel>
      <TabPanel>
        <div class="text-lg">
          Code Content for {{ props.componentIdentification.schemaName }}
        </div>
      </TabPanel>
    </template>
  </SiTabGroup>
</template>

<script setup lang="ts">
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import { TabPanel } from "@headlessui/vue";
import AttributeViewer from "@/organisms/AttributeViewer.vue";
import _ from "lodash";
import { ComponentIdentification } from "@/api/sdf/dal/component";

const props = defineProps<{
  componentIdentification: ComponentIdentification;
}>();
</script>
