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
    <StatusIndicatorIcon type="action" :status="binding.kind" tone="inherit" class="flex-none" />
    <div class="font-bold leading-normal">
      {{ binding.displayName || binding.name }}
    </div>

    <Icon v-if="addRequestStatus.isPending || removeRequestStatus.isPending" name="loader" class="ml-auto" size="sm" />
    <div
      v-else
      :class="
        clsx('ml-auto mr-2xs hover:underline font-bold select-none', themeClasses('text-action-500', 'text-action-300'))
      "
      @click.stop="onClickView"
    >
      view
    </div>
  </div>
</template>

<script setup lang="ts">
import * as _ from "lodash-es";
import clsx from "clsx";
import { computed } from "vue";
import { Icon, themeClasses, Toggle } from "@si/vue-lib/design-system";
import { useRouter } from "vue-router";
import { storeToRefs } from "pinia";
import { useActionsStore } from "@/store/actions.store";
import { Action } from "@/api/sdf/dal/func";
import { useViewsStore } from "@/store/views.store";
import { ComponentType } from "@/api/sdf/dal/schema";
import StatusIndicatorIcon from "../StatusIndicatorIcon.vue";
import { DiagramGroupData, DiagramNodeData } from "../ModelingDiagram/diagram_types";

interface BindingWithDisplayName extends Action {
  displayName?: string | null;
  name: string;
}

const props = defineProps<{
  component: DiagramGroupData | DiagramNodeData;
  binding: BindingWithDisplayName;
}>();

const viewStore = useViewsStore();
const { selectedComponent } = storeToRefs(viewStore);
const actionsStore = useActionsStore();
const router = useRouter();

const action = computed(() => {
  const a = actionsStore.listActionsByComponentId
    .get(props.component.def.id)
    .find((a) => a.prototypeId === props.binding?.actionPrototypeId);
  return a;
});

function clickHandler() {
  if (action.value?.id) {
    actionsStore.CANCEL([action.value.id]);
  } else if (props.binding?.actionPrototypeId) {
    actionsStore.ADD_ACTION(props.component.def.id, props.binding?.actionPrototypeId);
  }
}

function onClickView() {
  if (props.binding && selectedComponent.value?.def.componentType !== ComponentType.View) {
    router.push({
      name: "workspace-lab-assets",
      query: {
        s: `a_${selectedComponent.value?.def.schemaVariantId}|f_${props.binding.funcId}`,
      },
    });
  }
}

const addRequestStatus = actionsStore.getRequestStatus(
  "ADD_ACTION",
  props.component.def.id,
  props.binding?.actionPrototypeId,
);
const removeRequestStatus = actionsStore.getRequestStatus(
  "CANCEL",
  computed(() => action.value?.id),
  // ^ this won't accept [] which doesnt bode well
);
</script>
