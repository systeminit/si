<template>
  <div
    v-if="action"
    :class="
      clsx(
        'flex gap-xs p-2xs border-x border-b',
        themeClasses('border-neutral-200', 'border-neutral-600'),
      )
    "
    @click="clickHandler"
  >
    <Toggle :selected="isActive" :click="clickHandler" />
    <StatusIndicatorIcon type="action" :status="action?.name" tone="inherit" />
    <div class="font-bold">{{ action?.displayName }}</div>

    <Icon
      v-if="addRequestStatus.isPending || removeRequestStatus.isPending"
      name="loader"
      class="ml-auto"
      size="sm"
    />
  </div>
</template>

<script setup lang="ts">
import * as _ from "lodash-es";
import clsx from "clsx";
import { PropType, computed } from "vue";
import { Icon, themeClasses } from "@si/vue-lib/design-system";
import { ActionPrototypeId, useActionsStore } from "@/store/actions.store";
import { ComponentId } from "@/api/sdf/dal/component";
import StatusIndicatorIcon from "../StatusIndicatorIcon.vue";
import Toggle from "./Toggle.vue";

const props = defineProps({
  componentId: { type: String as PropType<ComponentId>, required: true },
  actionPrototypeId: {
    type: String as PropType<ActionPrototypeId>,
    required: true,
  },
});

const actionsStore = useActionsStore();

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
    actionsStore.CANCEL([action.value.actionInstanceId]);
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
  "CANCEL",
  computed(() => action.value?.actionInstanceId),
  // ^ this won't accept [] which doesnt bode well
);
</script>
