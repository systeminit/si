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
import { ActionId, ActionKind, ActionPrototypeId } from "@/api/sdf/dal/action";
import { FuncId } from "@/api/sdf/dal/func";
import { SchemaVariantId } from "@/api/sdf/dal/schema";
import { ComponentId } from "@/api/sdf/dal/component";
import ActionWidget from "./ActionWidget.vue";

const props = defineProps<{
  component: DiagramNodeData | DiagramGroupData;
  componentId: ComponentId;
}>();

const viewStore = useViewsStore();

const tabsRef = ref<InstanceType<typeof TabGroup>>();
function onTabSelected(newTabSlug?: string) {
  viewStore.setComponentDetailsTab(newTabSlug || null);
}

export interface ActionPrototypeView {
  id: ActionPrototypeId;
  funcId: FuncId;
  schemaVariantId: SchemaVariantId;
  kind: ActionKind;
  displayName?: string;
  name: string;
  actionId?: ActionId;
}

interface ActionPrototypeViewsByComponentId {
  id: ComponentId;
  actionPrototypes: ActionPrototypeView[];
}

const queryKey = makeKey(
  "ActionPrototypeViewsByComponentId",
  props.componentId,
);
const actionPrototypeViewsRaw =
  useQuery<ActionPrototypeViewsByComponentId | null>({
    queryKey,
    queryFn: async () =>
      await bifrost<ActionPrototypeViewsByComponentId>(
        makeArgs("ActionPrototypeViewsByComponentId", props.componentId),
      ),
  });
const actionPrototypeViews = computed(
  () => actionPrototypeViewsRaw.data.value?.actionPrototypes ?? [],
);

watch(
  () => viewStore.selectedComponentDetailsTab,
  (tabSlug) => {
    if (tabSlug?.startsWith("resource-")) {
      tabsRef.value?.selectTab(tabSlug);
    }
  },
);
</script>
