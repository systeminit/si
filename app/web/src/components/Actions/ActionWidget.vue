<template>
  <div
    v-if="binding"
    class="cursor-pointer"
    :class="
      clsx(
        'flex flex-row items-center gap-xs p-2xs border-x border-b',
        themeClasses('border-neutral-200', 'border-neutral-600'),
      )
    "
    @click="clickHandler"
  >
    <Toggle :selected="!!action" class="flex-none" />
    <StatusIndicatorIcon
      type="action"
      :status="binding.kind"
      tone="inherit"
      class="flex-none"
    />
    <div class="font-bold leading-normal">
      {{ binding.displayName || binding.name }}
    </div>
  </div>
</template>

<script setup lang="ts">
import * as _ from "lodash-es";
import clsx from "clsx";
import { computed } from "vue";
import { themeClasses, Toggle } from "@si/vue-lib/design-system";
import { useActionsStore } from "@/store/actions.store";
import { Action } from "@/api/sdf/dal/func";
import StatusIndicatorIcon from "../StatusIndicatorIcon.vue";
import {
  DiagramGroupData,
  DiagramNodeData,
} from "../ModelingDiagram/diagram_types";

interface BindingWithDisplayName extends Action {
  displayName?: string | null;
  name: string;
}

const props = defineProps<{
  component: DiagramGroupData | DiagramNodeData;
  binding: BindingWithDisplayName;
}>();

const actionsStore = useActionsStore();

const action = computed(() => {
  const a = actionsStore.listActionsByComponentId
    .get(props.component.def.id)
    .find((a) => a.prototypeId === props.binding?.actionPrototypeId);
  return a;
});

function clickHandler() {}
</script>
