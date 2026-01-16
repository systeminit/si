<template>
  <div class="h-full relative">
    <TabGroup
      variant="secondary"
      :startSelectedTabSlug="viewStore.detailsTabSlugs[1] || 'resource-actions'"
      marginTop="2xs"
      @update:selectedTab="onTabSelected"
    >
      <TabGroupItem label="Actions" slug="resource-actions">
        <div v-if="bindings.length === 0" class="flex flex-col items-center pt-lg h-full w-full text-neutral-400">
          <div class="w-64">
            <EmptyStateIcon name="no-changes" />
          </div>
          <span class="text-xl">No Actions available</span>
        </div>
        <div v-else class="flex flex-col">
          <div class="text-sm text-neutral-700 dark:text-neutral-300 p-xs italic border-b dark:border-neutral-600">
            The changes below will run when you click "Apply Changes".
          </div>
          <ActionWidget
            v-for="action in bindings"
            :key="action.actionPrototypeId || undefined"
            :binding="action"
            :component="props.component"
          />
        </div>
      </TabGroupItem>
      <TabGroupItem slug="resource-resource" label="Resource Data">
        <ComponentDetailsResource />
      </TabGroupItem>
    </TabGroup>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import * as _ from "lodash-es";
import { TabGroup, TabGroupItem } from "@si/vue-lib/design-system";
import { useFuncStore } from "@/store/func/funcs.store";
import EmptyStateIcon from "@/components/EmptyStateIcon.vue";
import ActionWidget from "@/components/Actions/ActionWidget.vue";
import { useViewsStore } from "@/store/views.store";
import ComponentDetailsResource from "./ComponentDetailsResource.vue";
import { DiagramGroupData, DiagramNodeData } from "./ModelingDiagram/diagram_types";

const props = defineProps<{
  component: DiagramNodeData | DiagramGroupData;
}>();

const funcStore = useFuncStore();
const viewStore = useViewsStore();

const tabsRef = ref<InstanceType<typeof TabGroup>>();
function onTabSelected(newTabSlug?: string) {
  viewStore.setComponentDetailsTab(newTabSlug || null);
}

const bindings = computed(() => funcStore.actionBindingsForSelectedComponent);

watch(
  () => viewStore.selectedComponentDetailsTab,
  (tabSlug) => {
    if (tabSlug?.startsWith("resource-")) {
      tabsRef.value?.selectTab(tabSlug);
    }
  },
);
</script>
