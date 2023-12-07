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
          <div :class="clsx('flex items-center mr-xs')">
            <!-- change status -->
            <StatusIndicatorIcon
              type="change"
              :status="hasChanges"
              @click.stop="onClick($event, 'diff')"
            />

            <!-- Qualification Status -->
            <!-- TODO: make click open details panel -->
            <Icon v-if="isDestroyed" name="none" />
            <StatusIndicatorIcon
              v-else
              type="qualification"
              :status="qualificationStatus || 'notexists'"
              @click.stop="onClick($event, 'qualifications')"
            />

            <!-- Resource Status -->

            <StatusIndicatorIcon
              v-if="hasResource"
              type="resource"
              status="exists"
              @click.stop="onClick($event, 'resource')"
            />
            <Icon v-else name="none" />

            <!-- Actions Menu
            <Icon name="chevron--right" /> -->
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
import { computed, PropType, ref, watch } from "vue";
import * as _ from "lodash-es";

import clsx from "clsx";
import { themeClasses, Icon, VButton } from "@si/vue-lib/design-system";
import { ComponentId, useComponentsStore } from "@/store/components.store";
import { useQualificationsStore } from "@/store/qualifications.store";

import { useChangeSetsStore } from "@/store/change_sets.store";
import ComponentOutlineNode from "./ComponentOutlineNode.vue"; // eslint-disable-line import/no-self-import
import StatusIndicatorIcon from "../StatusIndicatorIcon.vue";

import { useComponentOutlineContext } from "./ComponentOutline.vue";

const props = defineProps({
  componentId: { type: String as PropType<ComponentId>, required: true },
});

const rootCtx = useComponentOutlineContext();
const { filterModeActive } = rootCtx;

const nodeRef = ref<HTMLElement>();

const isOpen = ref(true);

const componentsStore = useComponentsStore();
const qualificationsStore = useQualificationsStore();
const changeSetsStore = useChangeSetsStore();

const component = computed(
  () => componentsStore.componentsById[props.componentId],
);

const hasResource = computed(() => component.value?.resource.data !== null);

const hasChanges = computed(() => component.value?.changeStatus);

const isDestroyed = computed(() => component.value?.changeStatus === "deleted");

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
  () => qualificationsStore.qualificationStatusByComponentId[props.componentId],
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

function onClick(e: MouseEvent, tabSlug?: string) {
  rootCtx.itemClickHandler(e, props.componentId, tabSlug);
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
</script>
