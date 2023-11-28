<template>
  <div
    v-if="action"
    :class="
      clsx(
        'flex items-center gap-xs p-2xs rounded-md cursor-pointer border',
        isActive ? 'bg-action-500 border-action-500 text-white' : '',
      )
    "
    @click="clickHandler"
  >
    <StatusIndicatorIcon type="action" :status="action?.name" tone="inherit" />
    <Stack spacing="2xs">
      <div>{{ action?.displayName }}</div>
      <div class="text-xs text-neutral-300">{{ component?.displayName }}</div>
    </Stack>

    <Icon
      v-if="addRequestStatus.isPending || removeRequestStatus.isPending"
      name="loader"
      class="ml-auto"
      size="sm"
    />
    <Icon v-else-if="isActive" name="x" class="ml-auto" size="sm" />
  </div>
</template>

<script setup lang="ts">
import * as _ from "lodash-es";
import clsx from "clsx";
import { PropType, computed } from "vue";
import { Icon, Stack } from "@si/vue-lib/design-system";
import { ActionPrototypeId, useActionsStore } from "@/store/actions.store";
import { ComponentId, useComponentsStore } from "@/store/components.store";
import StatusIndicatorIcon from "./StatusIndicatorIcon.vue";

const props = defineProps({
  componentId: { type: String as PropType<ComponentId>, required: true },
  actionPrototypeId: {
    type: String as PropType<ActionPrototypeId>,
    required: true,
  },
});

const actionsStore = useActionsStore();
const componentsStore = useComponentsStore();

const component = computed(
  () => componentsStore.componentsById[props.componentId],
);

const action = computed(() => {
  return _.find(
    actionsStore.actionsByComponentId[props.componentId],
    (a) => a.actionPrototypeId === props.actionPrototypeId,
  );
});

const isActive = computed(() => !!action.value?.actionInstanceId);

function clickHandler() {
  if (!action.value) return;
  if (action.value.actionInstanceId) {
    actionsStore.REMOVE_ACTION(action.value.actionInstanceId);
  } else {
    actionsStore.ADD_ACTION(props.componentId, props.actionPrototypeId);
  }
}

const addRequestStatus = actionsStore.getRequestStatus(
  "ADD_ACTION",
  props.componentId,
  props.actionPrototypeId,
);
const removeRequestStatus = actionsStore.getRequestStatus(
  "REMOVE_ACTION",
  computed(() => action.value?.actionInstanceId),
);
</script>
