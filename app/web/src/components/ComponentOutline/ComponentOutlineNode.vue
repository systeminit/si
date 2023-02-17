<template>
  <div class="">
    <!-- component info -->
    <div
      :class="
        clsx(
          'relative border-b border-l-[4px] cursor-pointer group',
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
            'text-[9px] capsize p-2xs',
            themeClasses(
              'bg-neutral-100 text-neutral-600',
              'bg-neutral-700 text-neutral-300',
            ),
          )
        "
      >
        {{ parentBreadcrumbsText }}
      </div>
      <div class="flex items-center p-xs">
        <Icon
          :name="component.icon"
          size="sm"
          :class="
            clsx(
              'mr-xs',
              enableGroupToggle && 'group-hover:scale-0 transition-all',
            )
          "
          :style="{ color: component.color }"
        />

        <div class="flex flex-col gap-[6px] select-none">
          <div class="capsize text-[11px] font-bold">
            {{ component.displayName }}
          </div>
          <div class="capsize text-[10px] italic">
            {{ component.schemaName }}
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

        <div class="ml-auto flex">
          <!-- other status icons -->
          <div
            v-if="component.changeStatus !== 'deleted'"
            class="flex mr-xs items-center"
          >
            <StatusIndicatorIcon
              type="qualification"
              :status="qualificationStatus"
              size="xs"
            />
            <StatusIndicatorIcon
              type="confirmation"
              :status="confirmationStatus"
              size="xs"
            />
          </div>

          <!-- change status -->
          <StatusIndicatorIcon
            type="change"
            :status="component.changeStatus"
            size="sm"
          />
        </div>
      </div>
    </div>
    <!-- children -->
    <div v-if="enableGroupToggle && isOpen" class="pl-xs">
      <ComponentOutlineNode
        v-for="child in childComponents"
        :key="child.id"
        :component-id="child.id"
      />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, PropType, ref } from "vue";
import _ from "lodash";

import clsx from "clsx";
import { ComponentId, useComponentsStore } from "@/store/components.store";

import Icon from "@/ui-lib/icons/Icon.vue";
import { themeClasses } from "@/ui-lib/theme_tools";
import { useQualificationsStore } from "@/store/qualifications.store";
import { useFixesStore } from "@/store/fixes.store";
import ComponentOutlineNode from "./ComponentOutlineNode.vue"; // eslint-disable-line import/no-self-import
import StatusIndicatorIcon from "../StatusIndicatorIcon.vue";
import { useComponentOutlineContext } from "./ComponentOutline.vue";

const props = defineProps({
  componentId: { type: String as PropType<ComponentId>, required: true },
});

const rootCtx = useComponentOutlineContext();
const { filterModeActive } = rootCtx;

const isOpen = ref(true);

const componentsStore = useComponentsStore();
const qualificationsStore = useQualificationsStore();
const fixesStore = useFixesStore();

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
    component.value.isGroup &&
    childComponents.value.length &&
    !filterModeActive.value,
);

const qualificationStatus = computed(
  () =>
    // qualificationStore.qualificationStatusWithRollupsByComponentId[
    qualificationsStore.qualificationStatusByComponentId[props.componentId],
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
  if (!component.value.parentId) return;

  const parentIds =
    componentsStore.parentIdPathByComponentId[component.value.id];
  return _.map(
    parentIds,
    (parentId) => componentsStore.componentsById[parentId].displayName,
  ).join(" > ");
});
</script>
