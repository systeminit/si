<template>
  <TreeNode
    leftBorderSize="xl"
    :color="theme === 'dark' ? '#525252' : '#E5E5E5'"
    staticContentClasses="border-b border-neutral-200 dark:border-neutral-600"
  >
    <template #label>
      <div class="flex flex-row items-center text-sm overflow-hidden w-full">
        <StatusIndicatorIcon type="action-state" :status="action.state" />
        <div class="flex flex-col pl-xs flex-shrink overflow-hidden flex-grow">
          <div>{{ `${action.name}` }}</div>
          <div
            ref="componentNameRef"
            v-tooltip="componentNameTooltip"
            :class="
              clsx(
                'truncate cursor-pointer',
                componentsStore.allComponentsById[props.action.componentId ?? -1]
                  ? 'dark:text-action-300 text-action-500 hover:underline font-bold'
                  : 'text-neutral-500 dark:text-neutral-400 line-through',
                isHover && 'underline',
              )
            "
            @click="onClick"
            @mouseenter="onHoverStart"
            @mouseleave="onHoverEnd"
          >
            {{ componentsStore.allComponentsById[props.action.componentId ?? -1]?.def.displayName ?? "unknown" }}
          </div>
        </div>
      </div>
    </template>
    <template #icons> </template>
    <template #staticContent> </template>
  </TreeNode>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import clsx from "clsx";
import { computed, ref } from "vue";
import { TreeNode, useTheme } from "@si/vue-lib/design-system";
import { ActionProposedView } from "@/store/actions.store";
import { useComponentsStore } from "@/store/components.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useViewsStore } from "@/store/views.store";
import StatusIndicatorIcon from "../StatusIndicatorIcon.vue";

const changeSetsStore = useChangeSetsStore();
const componentsStore = useComponentsStore();

const { theme } = useTheme();

const props = defineProps<{
  action: ActionProposedView;
}>();

const componentNameRef = ref();
const componentNameTooltip = computed(() => {
  if (componentNameRef.value) {
    if (!componentsStore.allComponentsById[props.action.componentId ?? -1]) {
      return {
        content: `Component "${componentNameRef.value.textContent}" does not exist ${
          changeSetsStore.headSelected ? "on head" : "in this change set"
        }.`,
        delay: { show: 700, hide: 10 },
      };
    } else if (componentNameRef.value.scrollWidth > componentNameRef.value.offsetWidth) {
      return {
        content: componentNameRef.value.textContent,
        delay: { show: 700, hide: 10 },
      };
    }
  }
  return {};
});

const viewStore = useViewsStore();

function onClick() {
  const component = componentsStore.allComponentsById[props.action.componentId || ""];
  if (component) {
    viewStore.setSelectedComponentId(props.action.componentId);
    componentsStore.eventBus.emit("panToComponent", {
      component,
      center: true,
    });
    onHoverEnd();
  }
}

const isHover = computed(() => viewStore.hoveredComponentId === props.action.componentId);

function onHoverStart() {
  if (props.action.componentId && componentsStore.allComponentsById[props.action.componentId]) {
    viewStore.setHoveredComponentId(props.action.componentId);
  }
}

function onHoverEnd() {
  if (props.action.componentId && componentsStore.allComponentsById[props.action.componentId]) {
    viewStore.setHoveredComponentId(null);
  }
}
</script>
