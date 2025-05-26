<template>
  <section :class="clsx('grid h-full', showGrid ? 'explore' : 'map')">
    <!-- Left column -->
    <!-- 12 pixel padding to align with the SI logo -->
    <div
      class="main pt-xs flex flex-col gap-xs items-stretch [&>div]:mx-[12px]"
    >
      <div class="flex-none flex flex-row items-center gap-xs">
        <TabGroupToggle ref="group" :aOrB="urlGridOrMap === 'grid'">
          <template #a="{ selected, toggle }">
            <VButton
              label="Grid"
              size="sm"
              variant="ghost"
              :tone="selected ? 'action' : 'shade'"
              @click="toggle"
            />
          </template>
          <template #b="{ selected, toggle }">
            <VButton
              label="Map"
              size="sm"
              variant="ghost"
              :tone="selected ? 'action' : 'shade'"
              @click="toggle"
            />
          </template>
        </TabGroupToggle>
        <DropdownMenuButton
          class="rounded min-w-[128px]"
          :options="viewListOptions"
          :modelValue="selectedView"
          placeholder="All Views"
          @update:modelValue="(val) => (selectedView = val)"
        >
          <template #afterOptions>
            <DropdownMenuItem
              label="Add a View"
              icon="plus"
              @select="openViewModal"
            />
          </template>
        </DropdownMenuButton>
        <InstructiveVormInput
          class="rounded grow"
          :activeClasses="
            clsx(themeClasses('border-action-500', 'border-action-300'))
          "
          inactiveClasses="border-neutral-500"
          :pills="['Up', 'Down', 'Left', 'Right']"
          instructions="to navigate"
        >
          <template #left>
            <Icon name="search" tone="neutral" size="sm" />
          </template>
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
      <template v-if="showGrid">
        <div ref="scrollRef" class="scrollable tilegrid grow">
          <ComponentGridTile
            v-for="component in componentVirtualItemsList"
            :key="filteredComponents[component.index]!.id"
            :class="clsx(tileClasses(component.index))"
            :component="filteredComponents[component.index]!"
            @dblclick="
              componentNavigate(filteredComponents[component.index]!.id)
            "
          />
          <div
            v-if="
              componentList.length === 0 && componentListRaw.isSuccess.value
            "
          >
            <em>No components in View</em>
          </div>
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
            @click="openAddComponentModal"
          />
        </footer>
      </template>
      <Map v-else :active="!showGrid" />
    </div>
    <!-- Right column -->
    <div
      :class="
        clsx(
          'right flex flex-col border-l border-neutral-500',
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
        <CollapsingGridItem ref="history" disableScroll>
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
    <AddComponentModal
      ref="addComponentModalRef"
      :viewId="selectedViewOrDefaultId"
    />
    <AddViewModal
      ref="addViewModalRef"
      :views="viewListQuery.data.value?.views"
    />
  </section>
</template>

<script lang="ts" setup>
import { computed, inject, provide, reactive, ref, watch } from "vue";
import { useRouter, useRoute } from "vue-router";
import {
  themeClasses,
  VormInput,
  VButton,
  DropdownMenuButton,
  DropdownMenuItem,
  Icon,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { useQuery } from "@tanstack/vue-query";
import { useVirtualizer } from "@tanstack/vue-virtual";
import { Fzf } from "fzf";
import { bifrost, useMakeArgs, useMakeKey } from "@/store/realtime/heimdall";
import {
  BifrostActionViewList,
  BifrostComponent,
  BifrostComponentList,
  BifrostViewList,
  ViewComponentList,
  EntityKind,
} from "@/workers/types/entity_kind_types";
import RealtimeStatusPageState from "@/components/RealtimeStatusPageState.vue";
import { ComponentId } from "@/api/sdf/dal/component";
import Map from "./Map.vue";
import { collapsingGridStyles } from "./util";
import CollapsingGridItem from "./layout_components/CollapsingGridItem.vue";
import InstructiveVormInput from "./layout_components/InstructiveVormInput.vue";
import ComponentGridTile from "./ComponentGridTile.vue";
import Breadcrumbs from "./layout_components/Breadcrumbs.vue";
import ActionCard from "./ActionCard.vue";
import FuncRunList from "./FuncRunList.vue";
import { assertIsDefined, Context } from "./types";
import { keyEmitter } from "./logic_composables/emitters";
import TabGroupToggle from "./layout_components/TabGroupToggle.vue";
import { SelectionsInQueryString } from "./Workspace.vue";
import AddComponentModal from "./AddComponentModal.vue";
import AddViewModal from "./AddViewModal.vue";

const router = useRouter();
const route = useRoute();
const ctx = inject<Context>("CONTEXT");
assertIsDefined(ctx);

const selectedView = ref("");
const group = ref<InstanceType<typeof TabGroupToggle>>();

const urlGridOrMap = computed(() => {
  const q: SelectionsInQueryString = router.currentRoute.value?.query;
  const keys = Object.keys(q);
  if (keys.includes("grid")) return "grid";
  if (keys.includes("map")) return "map";
  return "grid";
});
const showGrid = computed(() => group.value?.isA);
watch(showGrid, () => {
  const query: SelectionsInQueryString = {
    ...router.currentRoute.value?.query,
  };
  delete query.map;
  delete query.grid;
  if (showGrid.value) query.grid = "1";
  else query.map = "1";
  router.push({ query });
});

const actions = ref<typeof CollapsingGridItem>();
const history = ref<typeof CollapsingGridItem>();

const key = useMakeKey();
const args = useMakeArgs();

const viewListQuery = useQuery<BifrostViewList | null>({
  queryKey: key(EntityKind.ViewList),
  queryFn: async () =>
    await bifrost<BifrostViewList>(args(EntityKind.ViewList)),
});
const viewListOptions = computed(() => {
  const list = viewListQuery.data.value?.views || [];
  const options = [{ value: "", label: "All Views" }];
  return options.concat(
    list.map((l) => {
      return { value: l.id, label: l.name };
    }),
  );
});

const selectedViewOrDefaultId = computed(() => {
  if (selectedView.value) return selectedView.value;
  if (!viewListQuery.data.value) return "";
  const view = viewListQuery.data.value.views.find((v) => v.isDefault);
  if (!view) return "";
  return view.id;
});

const actionViewListRaw = useQuery<BifrostActionViewList | null>({
  queryKey: key(EntityKind.ActionViewList),
  queryFn: async () =>
    await bifrost<BifrostActionViewList>(args(EntityKind.ActionViewList)),
});
const actionViewList = computed(
  () => actionViewListRaw.data.value?.actions ?? [],
);

const kind = computed(() =>
  selectedView.value ? EntityKind.ViewComponentList : EntityKind.ComponentList,
);
const id = computed(() =>
  selectedView.value ? selectedView.value : ctx.changeSetId.value,
);
const componentQueryKey = key(kind, id);

const componentListRaw = useQuery<
  BifrostComponentList | ViewComponentList | null
>({
  queryKey: componentQueryKey,
  queryFn: async () => {
    const arg = selectedView.value
      ? args(EntityKind.ViewComponentList, selectedView.value)
      : args(EntityKind.ComponentList);
    return await bifrost<BifrostComponentList | ViewComponentList>(arg);
  },
});

const componentList = computed(
  () => componentListRaw.data.value?.components ?? [],
);

const scrollRef = ref<HTMLDivElement>();

const filteredComponents = reactive<BifrostComponent[]>([]);

const searchString = ref("");
const computedSearchString = computed(() => searchString.value);

// send this down to any components that might use it
provide("SEARCH", computedSearchString);

watch(
  () => [searchString.value, componentList.value],
  () => {
    if (!searchString.value) {
      filteredComponents.splice(0, Infinity, ...componentList.value);
      return;
    }

    const fzf = new Fzf(componentList.value, {
      casing: "case-insensitive",
      selector: (c) =>
        `${c.name} ${c.schemaVariantName} ${c.schemaName} ${c.schemaCategory} ${c.schemaId} ${c.id}`,
    });

    const results = fzf.find(searchString.value);
    filteredComponents.splice(0, Infinity, ...results.map((fz) => fz.item));
  },
  { immediate: true },
);

const lanes = computed(() =>
  // Our grid is based on a minimum 250px width tile... so how many tiles can we fit?
  Math.floor((scrollRef.value?.offsetWidth ?? 0) / 250),
);

const virtualizerOptions = computed(() => {
  return {
    count: filteredComponents.length,
    // `lanes` gives virtualizer a "second-dimension" (aka columns for vertical lists and rows for horizontal lists)
    // https://tanstack.com/virtual/latest/docs/api/virtualizer#lanes
    // Our grid is based on a minimum 250px width tile... so how many tiles can we fit?
    // thats the value of `lanes`
    lanes: lanes.value,
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

const collapsingStyles = computed(() =>
  collapsingGridStyles([actions.value?.openState, history.value?.openState]),
);

const selectedComponentIds = reactive<Set<string>>(new Set());
const selectorGridPosition = ref<number>(-1);
const constrainPosition = () => {
  selectorGridPosition.value = Math.min(
    filteredComponents.length - 1,
    Math.max(-1, selectorGridPosition.value),
  );
};
const isSelected = (idx: number) =>
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  selectedComponentIds.has(filteredComponents[idx]!.id);
const isHovered = (idx: number) => selectorGridPosition.value === idx;
const tileClasses = (idx: number) => {
  const selected = isSelected(idx);
  const hovered = isHovered(idx);
  if (hovered) return "border-white border-[1px]";
  else if (selected) return "border-action-400 border-[1px]";
  else return "";
};

const inputRef = ref<HTMLInputElement>();
keyEmitter.on("k", (e) => {
  if (e.metaKey || e.ctrlKey) {
    inputRef.value?.focus();
  }
});
keyEmitter.on("a", (e) => {
  if (e.metaKey || e.ctrlKey) {
    openAddComponentModal();
  }
});

keyEmitter.on("ArrowDown", () => {
  if (!showGrid.value) return;
  selectorGridPosition.value += lanes.value;
  constrainPosition();
});
keyEmitter.on("ArrowUp", () => {
  if (!showGrid.value) return;
  selectorGridPosition.value -= lanes.value;
  constrainPosition();
});
keyEmitter.on("ArrowLeft", () => {
  if (!showGrid.value) return;
  selectorGridPosition.value -= 1;
  constrainPosition();
});
keyEmitter.on("ArrowRight", () => {
  if (!showGrid.value) return;
  selectorGridPosition.value += 1;
  constrainPosition();
});
keyEmitter.on("Enter", () => {
  if (selectorGridPosition.value !== -1) {
    const componentId = filteredComponents[selectorGridPosition.value]?.id;
    if (!componentId) return;
    if (selectedComponentIds.has(componentId))
      selectedComponentIds.delete(componentId);
    else selectedComponentIds.add(componentId);
  }
});

const componentNavigate = (componentId: ComponentId) => {
  const params = { ...route.params };
  params.componentId = componentId;
  router.push({
    name: "new-hotness-component",
    params,
  });
};

const addComponentModalRef = ref<InstanceType<typeof AddComponentModal>>();

const openAddComponentModal = () => {
  addComponentModalRef.value?.open();
};

const addViewModalRef = ref<InstanceType<typeof AddViewModal>>();

const openViewModal = () => {
  addViewModalRef.value?.open();
};
</script>

<style lang="css" scoped>
section.grid.explore {
  grid-template-columns: minmax(0, 70%) minmax(0, 30%);
  grid-template-rows: 100%;
  grid-template-areas: "main right";
}

section.grid.map {
  grid-template-columns: 100%;
  grid-template-rows: 100%;
  grid-template-areas: "main";
}

div.main {
  grid-area: "main";
}
div.right {
  grid-area: "right";
}
</style>
