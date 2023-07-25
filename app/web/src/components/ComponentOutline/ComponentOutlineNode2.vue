<template>
  <div
    v-if="component"
    ref="nodeRef"
    class="component-outline-node"
    :data-component-id="componentId"
  >
    <!-- component info -->
    <div
      :class="
        clsx(
          'relative border-b border-l-[2px] cursor-pointer group',
          themeClasses('border-neutral-200', 'border-neutral-600'),
          isHover && 'outline-blue-300 outline z-10 -outline-offset-1',
          isSelected && themeClasses('bg-action-100', 'bg-action-900'),
        )
      "
      :style="{
        borderLeftColor: component.color,
        // backgroundColor: bodyBg,
      }"
      @click="onClick"
      @click.right="onClick"
      @dblclick="onClick"
      @mouseenter="onHoverStart"
      @mouseleave="onHoverEnd"
    >
      <!-- parent breadcrumbs (only shown in filtered mode) -->
      <div
        v-if="filterModeActive && parentBreadcrumbsText"
        :class="
          clsx(
            'text-[10px] capsize pl-xs flex items-center',
            themeClasses(
              'bg-neutral-100 text-neutral-600',
              'bg-neutral-700 text-neutral-300',
            ),
          )
        "
      >
        <Icon name="tree-parents" size="xs" class="mr-2xs" />
        {{ parentBreadcrumbsText }}
      </div>
      <div class="flex flex-row items-center px-xs w-full gap-1">
        <Icon
          :name="component.icon"
          size="sm"
          :class="
            clsx(
              'mr-xs flex-none',
              enableGroupToggle && 'group-hover:scale-0 transition-all',
            )
          "
        />

        <div class="flex flex-col select-none overflow-hidden py-xs">
          <div
            class="capsize text-[13px] font-bold relative leading-loose pb-xs"
          >
            <div class="truncate w-full">{{ component.displayName }}</div>
          </div>
          <div class="capsize text-[11px] italic relative">
            <div class="truncate w-full">{{ component.schemaName }}</div>
          </div>
        </div>
        <!-- group open/close controls -->
        <div
          v-if="enableGroupToggle"
          class="absolute left-[0px] cursor-pointer"
          @click="isOpen = !isOpen"
        >
          <Icon
            :name="isOpen ? 'chevron--down' : 'chevron--right'"
            size="lg"
            class="scale-[40%] translate-x-[-9px] translate-y-[13px] group-hover:scale-100 group-hover:translate-x-0 group-hover:translate-y-0 transition-all"
          />
        </div>

        <div class="ml-auto flex flex-none">
          <!-- refresh resource button -->
          <div class="pr-xs group-hover:block hidden">
            <VButton
              v-if="component.resource.data"
              icon="refresh"
              size="xs"
              variant="ghost"
              @click="componentsStore.REFRESH_RESOURCE_INFO(component!.id)"
            />
          </div>

          <!-- other status icons -->
          <div
            :class="
              clsx(
                'flex items-center',
                component.changeStatus === 'deleted' ? 'mr-1' : 'mr-xs',
              )
            "
          >
            <template v-if="component.changeStatus !== 'deleted'">
              <StatusIconWithPopover
                type="confirmation"
                :status="confirmationStatus"
                size="md"
                :popoverPosition="popoverPosition"
              >
                <div
                  class="bg-neutral-700 w-96 h-96 rounded flex flex-col overflow-clip text-white shadow-3xl dark"
                >
                  <div
                    class="bg-black uppercase font-bold text-md pt-sm pb-xs px-xs shrink-0"
                  >
                    <span>Changes</span>
                  </div>
                  <TabGroup as="template">
                    <TabList
                      class="bg-black flex px-2xs font-bold text-sm children:uppercase children:border-b children:border-transparent children:px-xs children:py-xs"
                    >
                      <Tab
                        class="ui-selected:border-action-300 ui-selected:text-action-300"
                      >
                        Proposed
                        <span
                          class="rounded-2xl ml-xs mr-xs px-2.5 border border-destructive-500 ui-selected:bg-destructive-500 ui-selected:text-neutral-900 text-destructive-500"
                        >
                          {{ recommendationsSelection.length }}
                        </span>
                      </Tab>
                      <Tab
                        class="ui-selected:border-action-300 ui-selected:text-action-300"
                      >
                        Applied
                        <span
                          class="rounded-2xl ml-xs mr-xs px-2.5 border border-success-500 ui-selected:bg-success-500 ui-selected:text-neutral-900 text-success-500"
                        >
                          {{ filteredBatches.length }}
                        </span>
                      </Tab>
                    </TabList>
                    <TabPanels as="template">
                      <TabPanel class="p-xs pb-0 overflow-auto grow">
                        <div
                          v-if="recommendationsSelection.length === 0"
                          class="flex flex-col items-center justify-center h-full w-full text-neutral-400 gap-sm"
                        >
                          <NoChanges />
                          <span class="text-xl">No Changes Proposed</span>
                        </div>
                        <ul v-else class="flex flex-col gap-2xs pb-xs">
                          <li class="py-xs px-sm text-sm">
                            Proposed Changes will be enacted upon click of the
                            <b>APPLY CHANGES</b> button in the right rail.
                          </li>
                          <li
                            v-for="(
                              { recommendation, selected }, key
                            ) in recommendationsSelection"
                            :key="key"
                            class="bg-black"
                          >
                            <RecommendationSprite
                              :key="key"
                              :recommendation="recommendation"
                              :selected="selected"
                              @click.stop
                              @toggle="
                                toggleRecommendation($event, recommendation)
                              "
                            />
                          </li>
                        </ul>
                      </TabPanel>
                      <TabPanel class="p-xs overflow-auto grow">
                        <div
                          v-if="filteredBatches.length === 0"
                          class="flex flex-col items-center justify-center h-full w-full text-neutral-400 gap-sm"
                        >
                          <NoChanges />
                          <span class="text-xl">No Changes Applied</span>
                        </div>
                        <ul v-else class="flex flex-col gap-2xs">
                          <li
                            v-for="(fixBatch, index) in filteredBatches"
                            :key="index"
                            class="bg-black p-xs"
                          >
                            <ApplyHistoryItem :fixBatch="fixBatch" />
                          </li>
                        </ul>
                      </TabPanel>
                    </TabPanels>
                  </TabGroup>
                </div>
              </StatusIconWithPopover>

              <div class="bg-neutral-500 w-[1px] h-4 mx-xs" />

              <StatusIconWithPopover
                type="qualification"
                :status="qualificationStatus"
                size="md"
                :popoverPosition="popoverPosition"
              >
                <div
                  class="bg-neutral-700 w-96 h-80 rounded flex flex-col overflow-clip text-white shadow-3xl dark"
                >
                  <div
                    class="bg-black uppercase font-bold text-md p-xs flex place-content-between items-center"
                  >
                    <span>Qualifications</span>
                    <div class="flex gap-xs p-2xs">
                      <span
                        v-if="qualificationsFailed"
                        class="flex items-center gap-0.5"
                      >
                        <StatusIndicatorIcon
                          class="inline-block"
                          type="qualification"
                          status="failure"
                          size="md"
                        />
                        {{ qualificationsFailed }}
                      </span>
                      <span
                        v-if="qualificationsWarned"
                        class="flex items-center gap-0.5"
                      >
                        <StatusIndicatorIcon
                          class="inline-block"
                          type="qualification"
                          status="warning"
                          size="md"
                        />
                        {{ qualificationsWarned }}
                      </span>
                      <span class="flex items-center gap-0.5">
                        <StatusIndicatorIcon
                          class="inline-block"
                          type="qualification"
                          status="success"
                          size="md"
                        />
                        {{ qualificationsSucceeded }}
                      </span>
                    </div>
                  </div>
                  <div class="p-xs pb-0 overflow-auto">
                    <div
                      v-for="(qualification, index) in componentQualifications"
                      :key="index"
                      class="basis-full lg:basis-1/2 xl:basis-1/3 overflow-hidden pb-xs"
                    >
                      <QualificationViewerSingle
                        :qualification="qualification"
                        :componentId="props.componentId"
                      />
                    </div>
                  </div>
                </div>
              </StatusIconWithPopover>
            </template>

            <!-- change status -->
            <StatusIndicatorIcon
              v-if="component.changeStatus === 'deleted'"
              type="change"
              :status="component.changeStatus"
              size="md"
            />
          </div>
        </div>
      </div>
    </div>
    <!-- children -->
    <div v-if="enableGroupToggle && isOpen" class="pl-xs">
      <ComponentOutlineNode
        v-for="child in childComponents"
        :key="child.id"
        :componentId="child.id"
      />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, PropType, ref, watch, onBeforeUnmount } from "vue";
import * as _ from "lodash-es";

import clsx from "clsx";
import { themeClasses, Icon, VButton } from "@si/vue-lib/design-system";
import { TabGroup, TabList, Tab, TabPanels, TabPanel } from "@headlessui/vue";
import { ComponentId, useComponentsStore } from "@/store/components.store";
import { useQualificationsStore } from "@/store/qualifications.store";
import { useFixesStore, Recommendation } from "@/store/fixes.store";
import StatusIconWithPopover from "@/components/ComponentOutline/StatusIconWithPopover.vue";
import QualificationViewerSingle from "@/components/StatusBarTabs/Qualification/QualificationViewerSingle.vue";
import { useChangeSetsStore } from "@/store/change_sets.store";
import NoChanges from "@/assets/images/no-changes.svg?component";
import RecommendationSprite from "@/components/RecommendationSprite2.vue";
import ApplyHistoryItem from "@/components/ApplyHistoryItem.vue";
import ComponentOutlineNode from "./ComponentOutlineNode2.vue"; // eslint-disable-line import/no-self-import
import StatusIndicatorIcon from "../StatusIndicatorIcon2.vue";

import { useComponentOutlineContext2 } from "./ComponentOutline2.vue";

const props = defineProps({
  componentId: { type: String as PropType<ComponentId>, required: true },
});

const rootCtx = useComponentOutlineContext2();
const { filterModeActive } = rootCtx;

const nodeRef = ref<HTMLElement>();

const isOpen = ref(true);

const componentsStore = useComponentsStore();
const qualificationsStore = useQualificationsStore();
const fixesStore = useFixesStore();
const changeSetsStore = useChangeSetsStore();

const component = computed(
  () => componentsStore.componentsById[props.componentId],
);
const childComponents = computed(
  () => componentsStore.componentsByParentId[props.componentId] || [],
);

const isSelected = computed(() =>
  componentsStore.selectedComponentIds.includes(props.componentId),
);

const enableGroupToggle = computed(
  () =>
    component.value?.isGroup &&
    childComponents.value.length &&
    !filterModeActive.value,
);

const qualificationStatus = computed(
  () =>
    // qualificationStore.qualificationStatusWithRollupsByComponentId[
    qualificationsStore.qualificationStatusByComponentId[props.componentId],
);
const qualificationStats = computed(
  () => qualificationsStore.qualificationStatsByComponentId[props.componentId],
);
const qualificationsFailed = computed(() =>
  qualificationStats.value ? qualificationStats.value.failed : 0,
);
const qualificationsWarned = computed(() =>
  qualificationStats.value ? qualificationStats.value.warned : 0,
);
const qualificationsSucceeded = computed(() =>
  qualificationStats.value ? qualificationStats.value.succeeded : 0,
);

watch(
  [
    () => changeSetsStore.selectedChangeSetWritten,
    () => qualificationsStore.checkedQualificationsAt,
  ],
  () => {
    qualificationsStore.FETCH_COMPONENT_QUALIFICATIONS(props.componentId);
  },
  { immediate: true },
);

const componentQualifications = computed(() =>
  // TODO remove clone and use toSorted when it gets widely supported
  _.clone(
    qualificationsStore.qualificationsByComponentId[props.componentId],
  )?.sort(({ result: a }, { result: b }) => {
    // non successful qualifications come first
    if (a?.status !== b?.status) {
      if (a?.status !== "success") {
        return -1;
      }
      if (b?.status !== "success") {
        return 1;
      }
    }
    return 0;
  }),
);

const confirmationStatus = computed(
  () => fixesStore.confirmationStatusByComponentId[props.componentId],
);

function onClick(e: MouseEvent) {
  rootCtx.itemClickHandler(e, props.componentId);
}

const isHover = computed(
  () => componentsStore.hoveredComponentId === props.componentId,
);

function onHoverStart() {
  componentsStore.setHoveredComponentId(props.componentId);
}

function onHoverEnd() {
  componentsStore.setHoveredComponentId(null);
}

const parentBreadcrumbsText = computed(() => {
  if (!component.value?.parentId) return;

  const parentIds =
    componentsStore.parentIdPathByComponentId[component.value.id];
  return _.map(
    parentIds,
    (parentId) => componentsStore.componentsById[parentId]?.displayName,
  ).join(" > ");
});

// POPOVER CODE
// Since we anchor the popover on the parent, for now it makes sense to have the position calculated on the parent
const popoverPosition = ref<{ x: number; y: number } | undefined>();
const popoverResize = _.debounce(() => {
  if (!nodeRef.value) {
    popoverPosition.value = undefined;
    return;
  }

  const nodeRect = nodeRef.value.getBoundingClientRect();
  popoverPosition.value = {
    x: Math.floor(nodeRect.right),
    y: Math.floor(nodeRect.top),
  };
}, 50);
const resizeObserver = new ResizeObserver(popoverResize);

watch(nodeRef, () => {
  if (nodeRef.value) {
    resizeObserver.observe(nodeRef.value);
  } else {
    resizeObserver.disconnect();
  }
});

onBeforeUnmount(() => {
  resizeObserver.disconnect();
});

const recommendationsSelection = computed(() =>
  _.filter(
    fixesStore.recommendationsSelection,
    (r) => r.recommendation.componentId === props.componentId,
  ),
);

const toggleRecommendation = (
  selected: boolean,
  recommendation: Recommendation,
) => {
  const key = `${recommendation.confirmationAttributeValueId}-${recommendation.actionKind}`;
  fixesStore.recommendationsSelection[key] = { recommendation, selected };
};

const fixBatches = computed(() => _.reverse([...fixesStore.fixBatches]));

const filteredBatches = computed(() =>
  fixBatches.value
    .map((batch) => ({
      ...batch,
      fixes: batch.fixes.filter((fix) => fix.componentId === props.componentId),
    }))
    .filter((batch) => batch.fixes.length),
);
</script>
