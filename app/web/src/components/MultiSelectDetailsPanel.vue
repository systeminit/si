<template>
  <div class="flex flex-col h-full w-full overflow-hidden">
    <div class="p-xs border-b dark:border-neutral-600 flex-none">
      <Inline alignY="center">
        <Icon size="md" name="multiselect" class="shrink-0 mr-2xs" />
        <div class="font-bold capsize">Multiple Components</div>
      </Inline>
    </div>

    <div class="overflow-y-auto">
      <div class="capsize p-xs mt-xs italic text-neutral-400 text-sm">
        {{ selectedComponentIds.length }} components selected:
      </div>
      <Stack spacing="xs" class="p-xs">
        <ComponentCard
          v-for="componentId in selectedComponentIds"
          :key="componentId"
          :componentId="componentId"
        />
      </Stack>
    </div>

    <!-- <div class="p-xs">
      <VButton
        v-if="selectedEdge.changeStatus !== 'deleted'"
        tone="destructive"
        icon="trash"
        label="Delete"
      />
    </div> -->
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed } from "vue";
import { Icon, Inline, Stack } from "@si/vue-lib/design-system";
import { useComponentsStore } from "@/store/components.store";
import ComponentCard from "./ComponentCard.vue";

const componentsStore = useComponentsStore();

const selectedComponentIds = computed(
  () => componentsStore.selectedComponentIds,
);
</script>
