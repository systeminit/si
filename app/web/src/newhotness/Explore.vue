<template>
  <section class="grid h-full">
    <!-- Left column -->
    <!-- 12 pixel padding to align with the SI logo -->
    <div class="pt-xs flex flex-col gap-xs items-stretch [&>div]:mx-[12px]">
      <div class="flex-none">
        <!-- TODO(Wendy) - search is not functional yet -->
        <!-- filter / top header -->
        <InstructiveVormInput
          class="rounded"
          :activeClasses="
            clsx(themeClasses('border-action-500', 'border-action-300'))
          "
          inactiveClasses="border-neutral-500"
          :pills="['Up', 'Down']"
          instructions="to navigate"
        >
          <template #default="slotProps">
            <VormInput
              ref="inputRef"
              v-model="searchString"
              autocomplete="off"
              :class="slotProps.class"
              noStyles
              placeholder="Search components"
              @focus="slotProps.focus"
              @blur="slotProps.blur"
            />
          </template>
        </InstructiveVormInput>
      </div>
      <div ref="scrollRef" class="scrollable tilegrid grow">
        <!-- body -->
        <ComponentGridTile
          v-for="component in componentVirtualItemsList"
          :key="componentList[component.index]!.id"
          :component="componentList[component.index]!"
          @dblclick="componentNavigate(componentList[component.index]!.id)"
        />
      </div>
      <footer
        :class="
          clsx(
            'flex-none h-12 p-2xs border-t border-neutral-500 flex flex-row justify-end items-center',
            themeClasses('bg-neutral-100', 'bg-neutral-800'),
          )
        "
      >
        <!-- footer -->
        <VButton
          label="Add a component"
          pill="Cmd + A"
          tone="action"
          size="sm"
        />
      </footer>
    </div>
    <!-- Right column -->
    <div
      :class="
        clsx(
          'flex flex-col border-l border-neutral-500',
          themeClasses('bg-neutral-100', 'bg-neutral-800'),
        )
      "
    >
      <!-- TODO(Wendy) - this section UI is still rough, see Figma -->
      <div class="grow grid grid-rows-subgrid" :style="collapsingStyles">
        <CollapsingGridItem ref="actions">
          <template #header>Actions ({{ actionViewList.length }})</template>
          <ul class="actions list">
            <ActionCard
              v-for="action in actionViewList"
              :key="action.id"
              :action="action"
              :selected="false"
              :noInteraction="false"
            />
          </ul>
        </CollapsingGridItem>
        <CollapsingGridItem ref="history">
          <template #header>History</template>
          <FuncRunList :limit="25" />
        </CollapsingGridItem>
      </div>
      <!-- TODO(Wendy) - moved this here for now, we can figure out the right spot later -->
      <div
        class="flex-none h-12 border-t border-neutral-500 flex flex-col justify-between p-2xs"
      >
        <Breadcrumbs class="text-xs" />
        <RealtimeStatusPageState />
      </div>
    </div>
  </section>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import { useRouter, useRoute } from "vue-router";
import { themeClasses, VormInput, VButton } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { useQuery } from "@tanstack/vue-query";
import { useVirtualizer } from "@tanstack/vue-virtual";
import { bifrost, makeArgs, makeKey } from "@/store/realtime/heimdall";
import {
  BifrostActionViewList,
  BifrostComponentList,
} from "@/workers/types/dbinterface";
import RealtimeStatusPageState from "@/components/RealtimeStatusPageState.vue";
import { ComponentId } from "@/api/sdf/dal/component";
import { collapsingGridStyles } from "./util";
import CollapsingGridItem from "./layout_components/CollapsingGridItem.vue";
import InstructiveVormInput from "./layout_components/InstructiveVormInput.vue";
import ComponentGridTile from "./ComponentGridTile.vue";
import Breadcrumbs from "./layout_components/Breadcrumbs.vue";
import ActionCard from "./ActionCard.vue";
import FuncRunList from "./FuncRunList.vue";

const actions = ref<typeof CollapsingGridItem>();
const history = ref<typeof CollapsingGridItem>();

const actionViewListRaw = useQuery<BifrostActionViewList | null>({
  queryKey: makeKey("ActionViewList"),
  queryFn: async () =>
    await bifrost<BifrostActionViewList>(makeArgs("ActionViewList")),
});
const actionViewList = computed(
  () => actionViewListRaw.data.value?.actions ?? [],
);

const componentListRaw = useQuery<BifrostComponentList | null>({
  queryKey: makeKey("ComponentList"),
  queryFn: async () =>
    await bifrost<BifrostComponentList>(makeArgs("ComponentList")),
});
const componentList = computed(
  () => componentListRaw.data.value?.components ?? [],
);

const scrollRef = ref<HTMLDivElement>();

const virtualizerOptions = computed(() => {
  return {
    count: componentList.value.length,
    // `lanes` gives virtualizer a "second-dimension" (aka columns for vertical lists and rows for horizontal lists)
    // https://tanstack.com/virtual/latest/docs/api/virtualizer#lanes
    // Our grid is based on a minimum 250px width tile... so how many tiles can we fit?
    // thats the value of `lanes`
    lanes: Math.floor(scrollRef.value?.offsetWidth ?? 0 / 250),
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    getScrollElement: () => scrollRef.value!,
    estimateSize: () => 200,
    overscan: 3,
  };
});

const virtualList = useVirtualizer(virtualizerOptions);

const componentVirtualItemsList = computed(() =>
  virtualList.value.getVirtualItems(),
);

const searchString = ref("");

const collapsingStyles = computed(() =>
  collapsingGridStyles([actions.value?.openState, history.value?.openState]),
);

const router = useRouter();
const route = useRoute();
const componentNavigate = (componentId: ComponentId) => {
  const params = { ...route.params };
  params.componentId = componentId;
  router.push({
    name: "new-hotness-component",
    params,
  });
};
</script>

<style lang="css" scoped>
section.grid {
  grid-template-columns: minmax(0, 70%) minmax(0, 30%);
  grid-template-rows: 100%;
}
</style>
