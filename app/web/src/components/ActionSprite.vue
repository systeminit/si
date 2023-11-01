<template>
  <div
    class="flex flex-row gap-2 items-center text-sm relative min-w-0 w-full justify-end p-xs"
    @click="addAction"
  >
    <StatusIndicatorIcon type="action" :status="action.name" tone="shade" />
    <div class="flex flex-col overflow-hidden">
      <div class="">{{ actionName }}</div>
      <div class="text-neutral-400 truncate">
        <!-- TODO(wendy) - sometimes the component name doesn't load properly? not sure why -->
        {{ component?.displayName ?? "unknown" }}
      </div>
      <div v-if="hasActor" class="text-neutral-400 truncate">
        By: {{ action.actor }}
      </div>
    </div>
    <VButton
      v-if="props.action.actionInstanceId"
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
import { useComponentsStore } from "@/store/components.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { ProposedAction } from "@/store/actions.store";
import StatusIndicatorIcon from "./StatusIndicatorIcon.vue";

const componentStore = useComponentsStore();
const changeSetStore = useChangeSetsStore();

const props = defineProps<{
  action: ProposedAction;
}>();

const actionName = computed(() => {
  const name = (props.action.displayName || props.action.name).trim();
  return name.length ? name.slice(0, 1).toUpperCase() + name.slice(1) : "";
});

const hasActor = computed(() => props.action.actor);

const component = computed(
  () => componentStore.componentsById[props.action.componentId],
);

const addAction = (event: Event) => {
  if (props.action.actionInstanceId || !changeSetStore.selectedChangeSet)
    return;
  event.preventDefault();
  event.stopPropagation();
  emit("add");
};
const removeAction = () => {
  if (!props.action.actionInstanceId || !changeSetStore.selectedChangeSet)
    return;
  emit("remove");
};

const emit = defineEmits<{
  (e: "add"): void;
  (e: "remove"): void;
}>();
</script>
