<template>
  <div
    class="flex flex-row gap-xs items-center text-sm relative p-xs min-w-0 w-full justify-end"
    @click="addAction"
  >
    <StatusIndicatorIcon type="action" :status="action.kind" tone="shade" />
    <div v-if="props.slim" class="flex flex-col overflow-hidden">
      {{ actionName }}
    </div>
    <div v-else class="flex flex-col overflow-hidden">
      <div class="">{{ actionName }}</div>
      <div
        :class="
          clsx(
            'truncate cursor-pointer ',
            component?.displayName
              ? 'dark:text-action-300 text-action-500 font-bold'
              : 'text-neutral-500 dark:text-neutral-400',
            isHover && 'underline',
          )
        "
        @click="onClick"
        @mouseenter="onHoverStart"
        @mouseleave="onHoverEnd"
      >
        {{ component?.displayName ?? "unknown" }}
      </div>
      <div
        v-if="hasActor"
        class="text-neutral-500 dark:text-neutral-400 truncate"
      >
        By: {{ action.actor }}
      </div>
    </div>
    <VButton
      v-if="props.action.id"
      class="ml-auto"
      size="xs"
      tone="shade"
      variant="transparent"
      icon="x"
      rounded
      @click.stop="removeAction"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";

import { VButton } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { useComponentsStore } from "@/store/components.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { DeprecatedProposedAction } from "@/store/actions.store";
import StatusIndicatorIcon from "./StatusIndicatorIcon.vue";

const componentsStore = useComponentsStore();
const changeSetStore = useChangeSetsStore();

const props = defineProps<{
  action: DeprecatedProposedAction;
  slim?: boolean;
}>();

const actionName = computed(() => {
  const name = (props.action.name || props.action.name).trim();
  return name.length ? name.slice(0, 1).toUpperCase() + name.slice(1) : "";
});

const hasActor = computed(() => props.action.actor);

const component = computed(
  () => componentsStore.componentsById[props.action.componentId],
);

const addAction = (event: Event) => {
  if (props.action.id || !changeSetStore.selectedChangeSet) return;
  event.preventDefault();
  event.stopPropagation();
  emit("add");
};
const removeAction = () => {
  if (!props.action.id || !changeSetStore.selectedChangeSet) return;
  emit("remove");
};

const emit = defineEmits<{
  (e: "add"): void;
  (e: "remove"): void;
}>();

function onClick() {
  if (componentsStore.componentsById[props.action.componentId]) {
    componentsStore.setSelectedComponentId(props.action.componentId);
    componentsStore.eventBus.emit("panToComponent", {
      componentId: props.action.componentId,
      center: true,
    });
  }
}

const isHover = computed(
  () => componentsStore.hoveredComponentId === props.action.componentId,
);

function onHoverStart() {
  if (componentsStore.componentsById[props.action.componentId]) {
    componentsStore.setHoveredComponentId(props.action.componentId);
  }
}

function onHoverEnd() {
  if (componentsStore.componentsById[props.action.componentId]) {
    componentsStore.setHoveredComponentId(null);
  }
}
</script>
