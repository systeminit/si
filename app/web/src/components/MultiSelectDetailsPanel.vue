<template>
  <div class="flex flex-col h-full">
    <div class="p-xs border-b dark:border-neutral-600">
      <Inline align-y="center">
        <Icon size="md" name="multiselect" class="shrink-0 mr-2xs" />
        <div class="font-bold capsize">Multiple Components</div>
      </Inline>
    </div>

    <div class="capsize p-xs mt-xs italic text-neutral-400 text-sm">
      {{ selectedComponentIds.length }} components selected:
    </div>
    <Stack spacing="xs" class="p-xs">
      <ComponentCard
        v-for="componentId in selectedComponentIds"
        :key="componentId"
        :component-id="componentId"
      />
    </Stack>

    <!-- <div class="p-xs">
      <VButton2
        v-if="selectedEdge.changeStatus !== 'deleted'"
        tone="destructive"
        icon="trash"
        label="Delete"
      />
    </div> -->
  </div>
</template>

<script lang="ts" setup>
import _ from "lodash";
import { TabPanel } from "@headlessui/vue";
import { computed, onBeforeMount } from "vue";
import { useComponentsStore } from "@/store/components.store";
import { useStatusStore } from "@/store/status.store";
import SiTabGroup from "@/components/SiTabGroup.vue";
import SiTabHeader from "@/components/SiTabHeader.vue";
import AttributeViewer from "@/components/AttributeViewer.vue";
import CodeViewer from "@/components/CodeViewer.vue";
import HealthIcon from "@/components/HealthIcon.vue";
import Timestamp from "@/ui-lib/Timestamp.vue";
import Icon from "@/ui-lib/icons/Icon.vue";
import ErrorMessage from "@/ui-lib/ErrorMessage.vue";
import VButton2 from "@/ui-lib/VButton2.vue";
import Inline from "@/ui-lib/layout/Inline.vue";
import Stack from "@/ui-lib/layout/Stack.vue";
import Divider from "@/ui-lib/layout/Divider.vue";
import ComponentCard from "./ComponentCard.vue";

const componentsStore = useComponentsStore();

const selectedComponentIds = computed(
  () => componentsStore.selectedComponentIds,
);
</script>
