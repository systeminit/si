<template>
  <div class="grid h-full grid-cols-2 gap-sm p-sm">
    <div
      :class="[
        'border flex flex-col gap-2xs',
        themeClasses('border-neutral-400', 'border-neutral-600'),
      ]"
    >
      <h3
        :class="
          clsx(
            'border-b',
            'flex-none flex items-center px-xs m-0 min-h-[2.5rem]',
            themeClasses(
              'bg-white border-neutral-400',
              'bg-neutral-800 border-neutral-600',
            ),
          )
        "
      >
        <span class="text-md font-bold">At A Glance:</span>
        <span
          :class="[
            'p-xs',
            themeClasses('border-neutral-300', 'border-neutral-700'),
          ]"
          >{{ changeSetName }}</span
        >
        <TextPill
          v-if="inReview"
          mono
          variant="simple"
          :class="
            clsx(
              'ml-auto border-0 text-xs',
              themeClasses(
                'bg-success-800 text-success-500',
                'bg-success-800 text-success-500',
              ),
            )
          "
        >
          In Review
        </TextPill>
      </h3>
      <div class="p-2xs pt-0 flex flex-col h-full">
        <div class="flex flex-col gap-sm text-sm p-sm">
          <div class="flex flex-col gap-md">
            <div class="flex flex-row gap-sm items-center">
              <div class="flex flex-row gap-xs items-center">
                <span>Explore components via</span>
                <ExploreModeTile icon="grid" label="Grid" @toggle="clickGrid" />
                <ExploreModeTile icon="map" label="Map" @toggle="clickMap" />
                <ExploreModeTile
                  v-if="!ctx.onHead.value"
                  icon="eye"
                  label="Review"
                  @toggle="clickReview"
                />
              </div>
            </div>

            <p v-if="ctx.onHead.value">
              HEAD represents the state of your infrastructure in the real
              world.
            </p>

            <DashboardDetails :details="changeSetDetails" />
          </div>
        </div>

        <CollapsingFlexItem open iconsLeft>
          <template #header>
            <span class="text-sm">Action Queue</span>
          </template>
          <template #headerIcons>
            <PillCounter :count="actionViewList.length" class="text-sm" />
          </template>
          <ActionQueueList
            :actionViewList="actionViewList"
            :highlightedActionIds="new Set()"
          />
        </CollapsingFlexItem>

        <CollapsingFlexItem open iconsLeft>
          <template #header>
            <span class="text-sm">Failed Qualifications</span>
          </template>
          <template #headerIcons>
            <PillCounter
              :count="componentWithFailedQual.length"
              class="text-sm"
            />
          </template>

          <div class="flex flex-row gap-sm p-sm">
            <ComponentCard
              v-for="component in componentWithFailedQual"
              :key="component.id"
              :component="component"
              class="cursor-pointer"
              @click="() => navToComponent(component)"
            />
          </div>
        </CollapsingFlexItem>
      </div>
    </div>

    <div
      :class="[
        'border flex flex-col gap-2xs',
        themeClasses('border-neutral-400', 'border-neutral-600'),
      ]"
    >
      <h3
        :class="
          clsx(
            'border-b',
            'flex-none flex items-center px-xs m-0 min-h-[2.5rem]',
            themeClasses(
              'bg-white border-neutral-400',
              'bg-neutral-800 border-neutral-600',
            ),
          )
        "
      >
        <span class="text-md font-bold">Simulated Change Sets</span>
      </h3>

      <div class="p-2xs pt-0 flex flex-col h-full">
        <CollapsingFlexItem
          :open="csForReview.length > 0"
          :disableOpen="csForReview.length === 0"
          iconsLeft
        >
          <template #header>
            <span class="text-sm">In Review</span>
          </template>
          <template #headerIcons>
            <PillCounter :count="csForReview.length" class="text-sm" />
          </template>
        </CollapsingFlexItem>
        <CollapsingFlexItem open iconsLeft>
          <template #header>
            <span class="text-sm">In Progress</span>
          </template>
          <template #headerIcons>
            <PillCounter :count="csInProgress.length" class="text-sm" />
          </template>

          <ul v-if="csInProgress.length > 0" class="m-sm flex flex-col gap-sm">
            <li
              v-for="cs in csInProgress"
              :key="cs.id"
              class="hover:bg-neutral-800 p-xs flex flex-col gap-sm cursor-pointer"
              @click="() => navToCS(cs)"
            >
              <div class="flex flex-row gap-sm">
                <span>{{ cs.name }}</span>
                <span>{{ cs.createdByUserId }}</span>
              </div>
              <DashboardDetails :size="'xs'" :details="detailById[cs.id]" />
            </li>
          </ul>

          <template v-if="csInProgress.length === 0">
            <p class="m-sm">
              Change Sets represent a batch of changes to the components,
              assets, functions, and actions in a workspace. When you want to
              propose a change in the real world, you first create a change set
              to do it. Nothing you do in a change set should alter the real
              world.
            </p>
            <p class="m-sm">
              If you are familiar with version control systems like git, you can
              think of a change set as an automatically rebasing branch.
            </p>
          </template>

          <NewButton
            icon="plus-circle"
            label="Create change set"
            tone="action"
            class="w-1/3 ml-auto mr-auto mt-md"
          />
        </CollapsingFlexItem>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import clsx from "clsx";
import {
  themeClasses,
  PillCounter,
  NewButton,
  TextPill,
} from "@si/vue-lib/design-system";
import { useRouter } from "vue-router";
import { useQueries, useQuery } from "@tanstack/vue-query";
import {
  BifrostActionViewList,
  ComponentInList,
  EntityKind,
} from "@/workers/types/entity_kind_types";
import {
  bifrost,
  bifrostList,
  useMakeArgs,
  useMakeKey,
} from "@/store/realtime/heimdall";
import { Listable } from "@/workers/types/dbinterface";
import { ChangeSet } from "@/api/sdf/dal/change_set";
import { rawUseMakeArgs, rawUseMakeKey } from "@/store/realtime/heimdall_inner";
import CollapsingFlexItem from "./layout_components/CollapsingFlexItem.vue";
import ExploreModeTile from "./ExploreModeTile.vue";
import ActionQueueList from "./ActionQueueList.vue";
import { useContext } from "./logic_composables/context";
import { useChangeSets } from "./logic_composables/change_set";
import { assertIsDefined, Context } from "./types";
import ComponentCard from "./ComponentCard.vue";
import DashboardDetails, {
  ChangeSetDetails,
} from "./layout_components/DashboardDetails.vue";

const router = useRouter();
const ctx = useContext();
assertIsDefined<Context>(ctx);

const { openChangeSets, changeSet: activeChangeSet } = useChangeSets(
  computed(() => ctx),
);

const changeSetName = computed(() => activeChangeSet.value?.name);

const clickMap = () => {
  const params = {
    ...router.currentRoute.value.params,
  };
  router.push({
    name: "new-hotness",
    params,
    query: { map: 1 },
  });
};

const clickGrid = () => {
  const params = {
    ...router.currentRoute.value.params,
  };
  router.push({
    name: "new-hotness",
    params,
    query: { grid: 1 },
  });
};

const clickReview = () => {
  const params = {
    ...router.currentRoute.value.params,
  };
  router.push({
    name: "new-hotness-review",
    params,
  });
};

const navToComponent = (component: ComponentInList) => {
  const params = {
    ...router.currentRoute.value.params,
    component: component.id,
  };
  router.push({
    name: "new-hotness",
    params,
  });
};

const navToCS = (cs: ChangeSet) => {
  const params = {
    ...router.currentRoute.value.params,
    changeSetId: cs.id,
  };
  router.push({
    name: "new-hotness-dash",
    params,
  });
};

const key = useMakeKey();
const args = useMakeArgs();
const actionViewListRaw = useQuery<BifrostActionViewList | null>({
  queryKey: key(EntityKind.ActionViewList),
  queryFn: async () =>
    await bifrost<BifrostActionViewList>(args(EntityKind.ActionViewList)),
});
const actionViewList = computed(
  () => actionViewListRaw.data.value?.actions ?? [],
);

const componentListQueryKind = computed(() => EntityKind.ComponentList);
const componentListQueryId = computed(() => ctx.workspacePk.value);
const componentQueryKey = key(componentListQueryKind, componentListQueryId);
const componentListQuery = useQuery<ComponentInList[]>({
  queryKey: componentQueryKey,
  enabled: ctx.queriesEnabled,
  queryFn: async () => {
    const arg = args<Listable>(EntityKind.ComponentList);
    const list = await bifrostList<ComponentInList[]>(arg);
    return list ?? [];
  },
});
const componentList = computed(() => {
  return componentListQuery.data.value ?? [];
});

const makeDetails = (listOfComponents: ComponentInList[]): ChangeSetDetails => {
  let components = 0;
  let resources = 0;
  let diff = 0;
  let failed = 0;
  listOfComponents.forEach((c) => {
    components += 1;
    if (c.hasResource) resources += 1;
    if (c.diffStatus !== "None") diff += 1;
    if (c.qualificationTotals.failed > 0) failed += 1;
  });
  return {
    components,
    resources,
    diff,
    failed,
  };
};
const changeSetDetails = computed<ChangeSetDetails>(() => {
  return makeDetails(componentList.value ?? []);
});

const componentWithFailedQual = computed(() =>
  componentList.value.filter((c) => c.qualificationTotals.failed > 0),
);

const csForReview = computed(() =>
  openChangeSets.value
    .filter((cs) => cs.mergeRequestedByUser && !cs.reviewedByUser)
    .filter((cs) => cs.id !== activeChangeSet.value?.id),
);

const csInProgress = computed(() =>
  openChangeSets.value
    .filter((cs) => !cs.mergeRequestedByUser)
    .filter((cs) => cs.id !== activeChangeSet.value?.id),
);

const inReview = computed(
  () =>
    activeChangeSet.value?.mergeRequestedByUser &&
    !activeChangeSet.value?.reviewedByUser,
);

const allCS = computed(() => [...csForReview.value, ...csInProgress.value]);
const detailsQueries = useQueries({
  queries: allCS.value.map((cs) => {
    return {
      queryKey: rawUseMakeKey({
        workspacePk: ctx.workspacePk,
        changeSetId: computed(() => cs.id),
      })(EntityKind.ComponentList),
      queryFn: async () => {
        const empty: ComponentInList[] = [];
        const arg = rawUseMakeArgs({
          workspacePk: ctx.workspacePk,
          changeSetId: computed(() => cs.id),
        })<Listable>(EntityKind.ComponentList);
        const list = await bifrostList<ComponentInList[]>(arg);
        return { changeSetId: cs.id, components: list ?? empty };
      },
    };
  }),
});

const detailById = computed(() => {
  const details: Record<string, ChangeSetDetails> = {};
  detailsQueries.value.forEach((query) => {
    if (!query.data) return;
    const { changeSetId, components } = query.data;
    if (!components || components.length === 0) return;
    details[changeSetId] = makeDetails(components);
  });
  return details;
});
</script>
