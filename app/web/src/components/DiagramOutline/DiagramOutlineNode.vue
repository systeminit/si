<template>
  <div
    v-if="component"
    :id="htmlid"
    ref="nodeRef"
    class="diagram-outline-node"
    :data-component-id="componentId"
  >
    <!-- component info -->
    <div
      :class="
        clsx(
          'relative border-b border-l-[2px] cursor-pointer group',
          themeClasses('border-neutral-200', 'border-neutral-600'),
          isHover &&
            'dark:outline-action-300 outline-action-500 outline z-10 -outline-offset-1 outline-1',
          isSelected && themeClasses('bg-action-100', 'bg-action-900'),
        )
      "
      :style="{
        borderLeftColor: component.color,
        // backgroundColor: bodyBg,
      }"
      @click="onClick"
      @dblclick="onClick"
      @contextmenu="onClick"
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
              v-if="component.hasResource"
              icon="refresh"
              size="xs"
              variant="ghost"
              @click="componentsStore.REFRESH_RESOURCE_INFO(component!.id)"
            />
          </div>

          <!-- other status icons -->
          <div :class="clsx('flex items-center mr-xs')">
            <div v-if="component.canBeUpgraded">
              <StatusIndicatorIcon
                class="hover:scale-110"
                size="sm"
                type="upgradable"
              />
            </div>

            <!-- change status -->
            <StatusIndicatorIcon
              class="hover:scale-110"
              size="sm"
              type="change"
              :status="hasChanges"
              @click.stop="onClick($event, 'diff')"
            />

            <!-- Qualification Status -->
            <!-- TODO: make click open details panel -->
            <Icon v-if="isDestroyed" name="none" size="sm" />
            <StatusIndicatorIcon
              v-else
              class="hover:scale-110"
              type="qualification"
              size="sm"
              :status="qualificationStatus || 'notexists'"
              @click.stop="onClick($event, 'qualifications')"
            />

            <!-- Resource Status -->

            <StatusIndicatorIcon
              v-if="component.hasResource"
              class="hover:scale-110"
              type="resource"
              status="exists"
              size="sm"
              @click.stop="onClick($event, 'resource')"
            />
            <Icon v-else name="none" size="sm" />

            <!-- Actions Menu
            <Icon name="chevron--right" /> -->
          </div>
        </div>
      </div>
    </div>
    <!-- children -->
    <div v-if="enableGroupToggle && isOpen" class="pl-xs">
      <DiagramOutlineNode
        v-for="child in childComponents"
        :key="child.id"
        :componentId="child.id"
      />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, PropType, ref } from "vue";
import * as _ from "lodash-es";

import clsx from "clsx";
import { themeClasses, Icon, VButton } from "@si/vue-lib/design-system";
import { useComponentsStore } from "@/store/components.store";
import { ComponentId } from "@/api/sdf/dal/component";
import { useQualificationsStore } from "@/store/qualifications.store";

import DiagramOutlineNode from "./DiagramOutlineNode.vue"; // eslint-disable-line import/no-self-import
import StatusIndicatorIcon from "../StatusIndicatorIcon.vue";

import { useDiagramOutlineContext } from "./DiagramOutline.vue";

const props = defineProps({
  componentId: { type: String as PropType<ComponentId>, required: true },
});

const rootCtx = useDiagramOutlineContext();
const { filterModeActive } = rootCtx;

const nodeRef = ref<HTMLElement>();
const htmlid = `diagram-outline-node-${props.componentId}`;

const isOpen = ref(true);

const componentsStore = useComponentsStore();
const qualificationsStore = useQualificationsStore();

const component = computed(
  () => componentsStore.componentsById[props.componentId],
);

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
