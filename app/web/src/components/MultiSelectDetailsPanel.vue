<template>
  <div class="flex flex-col h-full w-full overflow-hidden">
    <ScrollArea>
      <template #top>
        <SidebarSubpanelTitle label="Multiple Assets" icon="multiselect" />
      </template>

      <div class="capsize p-xs mt-xs italic text-neutral-400 text-sm">
        {{ selectedComponentIds.length }} assets selected:
      </div>
      <Stack spacing="xs" class="p-xs">
        <ComponentCard
          v-for="componentId in selectedComponentIds"
          :key="componentId"
          :componentId="componentId"
        />
      </Stack>
    </ScrollArea>
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed } from "vue";
import { ScrollArea, Stack } from "@si/vue-lib/design-system";
import { useComponentsStore } from "@/store/components.store";
import ComponentCard from "./ComponentCard.vue";
import SidebarSubpanelTitle from "./SidebarSubpanelTitle.vue";

const componentsStore = useComponentsStore();

const selectedComponentIds = computed(
  () => componentsStore.selectedComponentIds,
);
</script>
