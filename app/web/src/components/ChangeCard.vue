<template>
  <div
    :class="
      clsx(
        'flex flex-row gap-xs items-center text-sm relative p-xs min-w-0 w-full border-b',
        themeClasses('border-neutral-200', 'border-neutral-600'),
      )
    "
  >
    <StatusIndicatorIcon type="change" :status="diff.status" tone="shade" />
    <div class="flex flex-col overflow-hidden">
      <div class="">
        <span v-if="diff.status === 'added'">Added</span>
        <span v-if="diff.status === 'deleted'">Removed</span>
        <span v-if="diff.status === 'modified'">Modified</span>
        {{ componentsStore.allComponentsById[diff.componentId]?.def.schemaName }}
      </div>
      <div
        :class="clsx('dark:text-action-300 text-action-500 truncate cursor-pointer font-bold', isHover && 'underline')"
        @click="onClick"
        @mouseenter="onHoverStart"
        @mouseleave="onHoverEnd"
      >
        {{ componentsStore.allComponentsById[diff.componentId]?.def.displayName }}
      </div>
      <div class="text-neutral-500 dark:text-neutral-400 truncate">By: {{ diff.actor }}</div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { PropType, computed } from "vue";
import { ChangeStatus } from "@/api/sdf/dal/change_set";
import { useComponentsStore } from "@/store/components.store";
import { useViewsStore } from "@/store/views.store";
import StatusIndicatorIcon from "./StatusIndicatorIcon.vue";

export type DiffInfo = {
  componentId: string;
  status: ChangeStatus;
  updatedAt: string;
  actor: string;
};

const componentsStore = useComponentsStore();
const viewStore = useViewsStore();

const props = defineProps({
  diff: { type: Object as PropType<DiffInfo>, required: true },
});

function onClick() {
  const component = componentsStore.allComponentsById[props.diff.componentId];
  if (component) {
    viewStore.setSelectedComponentId(props.diff.componentId);
    componentsStore.eventBus.emit("panToComponent", {
      component,
      center: true,
    });
  }
}

const isHover = computed(() => viewStore.hoveredComponentId === props.diff.componentId);

function onHoverStart() {
  if (componentsStore.allComponentsById[props.diff.componentId]) {
    viewStore.setHoveredComponentId(props.diff.componentId);
  }
}

function onHoverEnd() {
  if (componentsStore.allComponentsById[props.diff.componentId]) {
    viewStore.setHoveredComponentId(null);
  }
}
</script>
