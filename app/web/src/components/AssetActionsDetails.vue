<template>
  <div class="h-full relative">
    <TabGroup
      variant="minimal"
      :startSelectedTabSlug="componentsStore.detailsTabSlugs[1] || undefined"
      marginTop="2xs"
      @update:selectedTab="onTabSelected"
    >
      <TabGroupItem label="Select" slug="actions-selection">
        <div
          v-if="bindings.length === 0"
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
            v-for="action in bindings"
            :key="action.actionPrototypeId || undefined"
            :binding="action"
            :componentId="props.componentId"
          />
        </div>
      </TabGroupItem>
      <TabGroupItem slug="actions-history">
        <template #label>
          <Inline>
            <span>History</span>
            <!-- <PillCounter class="ml-2xs" :count="filteredBatches.length" /> -->
          </Inline>
        </template>
      </TabGroupItem>
    </TabGroup>
  </div>
</template>

<script setup lang="ts">
import { computed, PropType, ref, watch } from "vue";
import * as _ from "lodash-es";

import { Inline, TabGroup, TabGroupItem } from "@si/vue-lib/design-system";

import { useComponentsStore } from "@/store/components.store";
import { ComponentId } from "@/api/sdf/dal/component";

import { Action } from "@/api/sdf/dal/func";
import { useFuncStore } from "@/store/func/funcs.store";
import EmptyStateIcon from "@/components/EmptyStateIcon.vue";
import ActionWidget from "@/components/Actions/ActionWidget.vue";

const props = defineProps({
  componentId: { type: String as PropType<ComponentId>, required: true },
});

const funcStore = useFuncStore();
const componentsStore = useComponentsStore();

const tabsRef = ref<InstanceType<typeof TabGroup>>();
function onTabSelected(newTabSlug?: string) {
  componentsStore.setComponentDetailsTab(newTabSlug || null);
}

interface BindingWithDisplayName extends Action {
  displayName: string;
}

const bindings = computed(() => {
  const _bindings = [] as BindingWithDisplayName[];
  const variant =
    componentsStore.schemaVariantsById[
      componentsStore.selectedComponent?.schemaVariantId || ""
    ];
  variant?.funcIds.forEach((funcId) => {
    const summary = funcStore.funcsById[funcId];
    const actions = funcStore.actionBindings[funcId]?.filter(
      (b) =>
        b.schemaVariantId ===
        componentsStore.selectedComponent?.schemaVariantId,
    );
    if (actions && actions.length > 0) {
      actions.forEach((b) => {
        const a = _.cloneDeep(b) as BindingWithDisplayName;
        a.displayName = summary?.displayName || "<Unknown>";
        _bindings.push(a);
      });
    }
  });
  return _bindings.sort((_a, _b) =>
    _a.displayName.localeCompare(_b.displayName),
  );
});

watch(
  () => componentsStore.selectedComponentDetailsTab,
  (tabSlug) => {
    if (tabSlug?.startsWith("actions")) {
      tabsRef.value?.selectTab(tabSlug);
    }
  },
);
</script>
