<template>
  <div
    v-if="binding"
    class="cursor-pointer"
    :class="
      clsx(
        'flex gap-xs p-2xs border-x border-b',
        themeClasses('border-neutral-200', 'border-neutral-600'),
      )
    "
    @click="clickHandler"
  >
    <Toggle :selected="!!action" :click="() => {}" />
    <StatusIndicatorIcon type="action" :status="binding.kind" tone="inherit" />
    <div class="font-bold">{{ binding.displayName }}</div>

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
import { useActionsStore } from "@/store/actions.store";
import { ComponentId } from "@/api/sdf/dal/component";
import { Action } from "@/api/sdf/dal/func";
import StatusIndicatorIcon from "../StatusIndicatorIcon.vue";
import Toggle from "./Toggle.vue";

interface BindingWithDisplayName extends Action {
  displayName: string;
}

const props = defineProps({
  componentId: { type: String as PropType<ComponentId>, required: true },
  binding: { type: Object as PropType<BindingWithDisplayName> },
});

const actionsStore = useActionsStore();

const action = computed(() => {
  const a = actionsStore.listActionsByComponentId
    .get(props.componentId)
    .find((a) => a.prototypeId === props.binding?.actionPrototypeId);
  return a;
});

function clickHandler() {
  if (action.value?.id) {
    actionsStore.CANCEL([action.value.id]);
  } else if (props.binding?.actionPrototypeId) {
    actionsStore.ADD_ACTION(
      props.componentId,
      props.binding?.actionPrototypeId,
    );
  }
}

const addRequestStatus = actionsStore.getRequestStatus(
  "ADD_ACTION",
  props.componentId,
  props.binding?.actionPrototypeId,
);
const removeRequestStatus = actionsStore.getRequestStatus(
  "CANCEL",
  computed(() => action.value?.id),
  // ^ this won't accept [] which doesnt bode well
);
</script>
