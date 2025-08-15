<template>
  <section
    v-if="ctx.onHead.value"
    class="p-lg flex flex-col items-center gap-sm"
  >
    <EmptyState
      icon="x"
      text="You are on HEAD"
      secondaryText="There are no changes to review"
    />
    <VButton
      label="Exit"
      tone="neutral"
      icon="chevron--left"
      @click="exitReview"
    />
  </section>
  <section v-else class="grid review w-full h-full">
    <div
      :class="
        clsx(
          'left',
          'flex flex-col border-r',
          themeClasses('border-neutral-400', 'border-neutral-600'),
        )
      "
    >
      <div
        :class="
          clsx(
            'h-xl flex-none w-full border-b flex flex-row items-center justify-center gap-xs [&>*]:font-bold',
            themeClasses('border-neutral-400', 'border-neutral-600'),
          )
        "
      >
        <PillCounter :count="changedComponents.length" size="lg" hideIfZero />
        <div class="text-xl text-center">Changed Components</div>
      </div>
      <div class="scrollable">
        <div
          v-for="component in changedComponents"
          :key="component.id"
          :class="
            clsx(
              'cursor-pointer border border-transparent',
              themeClasses(
                'hover:border-action-500',
                'hover:border-action-300',
              ),
            )
          "
          @click="scrollToDiff(component.id)"
        >
          <ComponentCard :component="component" />
        </div>
      </div>
    </div>
    <div class="main flex flex-col">
      <div
        :class="
          clsx(
            'flex flex-row flex-none items-center w-full p-xs border-b h-xl',
            themeClasses('border-neutral-400', 'border-neutral-600'),
          )
        "
      >
        <VButton
          label="Exit"
          tone="neutral"
          icon="chevron--left"
          class="flex-none"
          @click="exitReview"
        />
        <div class="flex-1 flex flex-col gap-xs p-xs text-center items-center">
          <div>
            Summary of changes for change set
            <span
              v-if="changeSetName"
              class="font-bold font-mono basis-0 flex-grow text-xs"
              >"{{ changeSetName }}"</span
            >
          </div>
          <div class="flex flex-row gap-xs">
            <div>{{ addedCount }} added</div>
            <div>{{ updatedCount }} updated</div>
            <div>{{ removedCount }} removed</div>
          </div>
        </div>
      </div>
      <div ref="mainScrollDivRef" class="scrollable flex flex-col gap-xs px-xs">
        <div
          v-for="component in changedComponents"
          :key="component.id"
          ref="componentDiffRefs"
          :data-component-id="component.id"
        >
          <ComponentCard :component="component" />
          <CodeViewer
            v-if="component.diffStatus === 'Added' || component.diffStatus === 'Modified'"
            :code="addedOrModifiedComponentData[component.id]?.data?.resourceDiff.diff"
            codeLanguage="diff"
            copyTooltip="Copy diff to clipboard"
          />
          <div v-else>{{ component.diffStatus }}</div>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import * as _ from "lodash-es";
import { useQueries, useQuery } from "@tanstack/vue-query";
import { computed, ref } from "vue";
import clsx from "clsx";
import { PillCounter, themeClasses, VButton } from "@si/vue-lib/design-system";
import { useRouter } from "vue-router";
import {
  bifrost,
  bifrostList,
  useMakeArgs,
  useMakeArgsForHead,
  useMakeKey,
  useMakeKeyForHead,
} from "@/store/realtime/heimdall";
import {
  BifrostComponent,
  ComponentInList,
  EntityKind,
} from "@/workers/types/entity_kind_types";
import CodeViewer from "@/components/CodeViewer.vue";
import { ComponentId } from "@/api/sdf/dal/component";
import ComponentCard from "./ComponentCard.vue";
import { useContext } from "./logic_composables/context";
import EmptyState from "./EmptyState.vue";

const router = useRouter();
const ctx = useContext();

const changeSetName = computed(() => ctx.changeSet.value?.name);

const headKey = useMakeKeyForHead();
const headArgs = useMakeArgsForHead();

// All components on HEAD
const headComponentListQuery = useQuery({
  queryKey: headKey(EntityKind.ComponentList),
  enabled: ctx.queriesEnabled,
  queryFn: async () => {
    console.log("head!", headArgs(EntityKind.ComponentList));
    const result = await bifrostList<ComponentInList[]>(headArgs(EntityKind.ComponentList));
    console.log("result", result);
    return result;
  }
});
const headComponentList = computed(() => headComponentListQuery.data.value ?? []);


const key = useMakeKey();
const args = useMakeArgs();

// All components on this change set
const componentListQuery = useQuery({
  queryKey: key(EntityKind.ComponentList),
  enabled: ctx.queriesEnabled,
  queryFn: async () =>
    await bifrostList<ComponentInList[]>(args(EntityKind.ComponentList)),
});
const componentList = computed(() => componentListQuery.data.value ?? []);

// Components in head but not this change set
const removedComponents = computed(() => {
  const componentIds = new Set(componentList.value.map((c) => c.id));
  return headComponentList.value.filter(
    (component) => !componentIds.has(component.id)
  ).map((component) => ({ ...component, diffStatus: "Removed" as const }));
});

// Components that are added or modified in the current change set
const addedOrModifiedComponents = computed(() =>
  componentList.value.filter((component) => component.diffStatus !== "None"),
);

const addedOrModifiedComponentDataQueries = useQueries({
  queries: computed(() =>
    addedOrModifiedComponents.value.map((component) => ({
      queryKey: key(EntityKind.Component, component.id),
      queryFn: async () =>
        await bifrost<BifrostComponent>(
          args(EntityKind.Component, component.id),
        ),
    })),
  ),
});
// Component MV data for added or modified components
const addedOrModifiedComponentData = computed(
  () => _.keyBy(addedOrModifiedComponentDataQueries.value, (component) => component.data?.id ?? ""),
);

// All differences from head (removals, additions and modifications)
const changedComponents = computed(() => [ ...addedOrModifiedComponents.value, ...removedComponents.value]);

const addedCount = computed(
  () =>
    changedComponents.value.filter(
      (component) => component.diffStatus === "Added",
    ).length,
);
const updatedCount = computed(
  () =>
    changedComponents.value.filter(
      (component) => component.diffStatus === "Modified",
    ).length,
);
const removedCount = computed(() => 
    changedComponents.value.filter(
      (component) => component.diffStatus === "Removed",
    ).length,
);

const mainScrollDivRef = ref<HTMLDivElement>();
const componentDiffRefs = ref<HTMLDivElement[]>();

const scrollToDiff = (componentId: ComponentId) => {
  const divEl = componentDiffRefs.value?.find(
    (divEl) => divEl.dataset.componentId === componentId,
  );
  divEl?.scrollIntoView({ behavior: "smooth" });
};

const exitReview = () => {
  router.push({
    name: "new-hotness",
  });
};
</script>

<style lang="css" scoped>
section.grid.review {
  grid-template-columns: minmax(0, 20%) minmax(0, 80%);
  grid-template-rows: 100%;
  grid-template-areas: "left main";
}

div.main {
  grid-area: "main";
}
div.left {
  grid-area: "left";
}
</style>
