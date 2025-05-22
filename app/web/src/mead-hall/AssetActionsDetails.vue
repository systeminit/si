<template>
  <div class="h-full relative">
    <TabGroup
      variant="secondary"
      :startSelectedTabSlug="viewStore.detailsTabSlugs[1] || 'resource-actions'"
      marginTop="2xs"
      @update:selectedTab="onTabSelected"
    >
      <TabGroupItem label="Actions" slug="resource-actions">
        <div
          v-if="actionPrototypeViews.length === 0"
          class="flex flex-col items-center pt-lg h-full w-full text-neutral-400"
        >
          <div class="w-64">
            <EmptyStateIcon name="no-changes" />
          </div>
          <span class="text-xl">No Actions available</span>
        </div>
        <div v-else class="flex flex-col">
          <div
            class="text-sm text-neutral-700 dark:text-neutral-300 p-xs italic border-b dark:border-neutral-600"
          >
            The changes below will run when you click "Apply Changes".
          </div>
          <ActionWidget
            v-for="actionPrototypeView in actionPrototypeViews"
            :key="actionPrototypeView.id"
            :actionPrototypeView="actionPrototypeView"
            :actionId="actionByPrototype[actionPrototypeView.id]"
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
import { useQuery } from "@tanstack/vue-query";
import { useViewsStore } from "@/store/views.store";
import ComponentDetailsResource from "@/components/ComponentDetailsResource.vue";
import {
  DiagramGroupData,
  DiagramNodeData,
} from "@/components/ModelingDiagram/diagram_types";
import { bifrost, makeArgs, makeKey } from "@/store/realtime/heimdall";
import EmptyStateIcon from "@/components/EmptyStateIcon.vue";
import { ActionPrototypeId, ActionId } from "@/api/sdf/dal/action";
import {
  ActionPrototypeViewList,
  BifrostActionViewList,
} from "@/workers/types/entity_kind_types";
import ActionWidget from "./ActionWidget.vue";

const props = defineProps<{
  component: DiagramNodeData | DiagramGroupData;
}>();

const viewStore = useViewsStore();

const tabsRef = ref<InstanceType<typeof TabGroup>>();
function onTabSelected(newTabSlug?: string) {
  viewStore.setComponentDetailsTab(newTabSlug || null);
}

// This is the core materialized view for this component. We need it to know what action prototypes
// are available for the given component.
const queryKeyForActionPrototypeViews = makeKey(
  "ActionPrototypeViewList",
  props.component.def.schemaVariantId,
);
const actionPrototypeViewsRaw = useQuery<ActionPrototypeViewList | null>({
  queryKey: queryKeyForActionPrototypeViews,
  queryFn: async () =>
    await bifrost<ActionPrototypeViewList>(
      makeArgs("ActionPrototypeViewList", props.component.def.schemaVariantId),
    ),
});
const actionPrototypeViews = computed(
  () => actionPrototypeViewsRaw.data.value?.actionPrototypes ?? [],
);

// Use the materialized view for actions to know what actions exist for a given prototype and the
// selected component.
const queryKeyForActionViewList = makeKey("ActionViewList");
const actionViewList = useQuery<BifrostActionViewList | null>({
  queryKey: queryKeyForActionViewList,
  queryFn: async () =>
    await bifrost<BifrostActionViewList>(makeArgs("ActionViewList")),
});
const actionByPrototype = computed(() => {
  if (!actionViewList.data.value?.actions) return {};
  if (actionViewList.data.value.actions.length < 1) return {};

  const result: Record<ActionPrototypeId, ActionId> = {};
  for (const action of actionViewList.data.value.actions) {
    if (action.componentId === props.component.def.id) {
      // NOTE(nick): this assumes that there can be one action for a given prototype and component.
      // As of the time of writing, this is true, but multiple actions per prototype and component
      // aren't disallowed from the underlying graph's perspective. Theorhetically, you could
      // enqueue two refreshes back-to-back. What then? I don't think we'll expose an interface to
      // do that for awhile, so this should be sufficient.
      result[action.prototypeId] = action.id;
    }
  }
  return result;
});

watch(
  () => viewStore.selectedComponentDetailsTab,
  (tabSlug) => {
    if (tabSlug?.startsWith("resource-")) {
      tabsRef.value?.selectTab(tabSlug);
    }
  },
);
</script>
