<template>
  <div
    class="cursor-pointer"
    :class="
      clsx(
        'flex flex-row items-center gap-xs p-2xs border-x border-b',
        themeClasses('border-neutral-200', 'border-neutral-600'),
      )
    "
    @click="clickHandler"
  >
    <Toggle :selected="!!actionPrototypeView.actionId" class="flex-none" />
    <StatusIndicatorIcon
      type="action"
      :status="FuncKind.Action"
      tone="inherit"
      class="flex-none"
    />
    <div class="font-bold leading-normal">
      {{ actionPrototypeView.displayName || actionPrototypeView.name }}
    </div>

    <Icon
      v-if="addRequestStatus.isPending || removeRequestStatus.isPending"
      name="loader"
      class="ml-auto"
      size="sm"
    />
    <div
      v-else
      :class="
        clsx(
          'ml-auto mr-2xs hover:underline font-bold select-none',
          themeClasses('text-action-500', 'text-action-300'),
        )
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
import { useViewsStore } from "@/store/views.store";
import { ComponentType } from "@/api/sdf/dal/schema";
import StatusIndicatorIcon from "@/components/StatusIndicatorIcon.vue";
import {
  DiagramGroupData,
  DiagramNodeData,
} from "@/components/ModelingDiagram/diagram_types";
import { FuncKind } from "@/api/sdf/dal/func";
import { ActionPrototypeView } from "./AssetActionsDetails.vue";

const props = defineProps<{
  component: DiagramGroupData | DiagramNodeData;
  actionPrototypeView: ActionPrototypeView;
}>();

const viewStore = useViewsStore();
const { selectedComponent } = storeToRefs(viewStore);
const actionsStore = useActionsStore();
const router = useRouter();

function clickHandler() {
  if (props.actionPrototypeView.actionId) {
    actionsStore.CANCEL([props.actionPrototypeView.actionId]);
  } else {
    actionsStore.ADD_ACTION(
      props.component.def.id,
      props.actionPrototypeView.id,
    );
  }
}

function onClickView() {
  if (selectedComponent.value?.def.componentType !== ComponentType.View) {
    router.push({
      name: "workspace-lab-assets",
      query: {
        s: `a_${selectedComponent.value?.def.schemaVariantId}|f_${props.actionPrototypeView.funcId}`,
      },
    });
  }
}

const addRequestStatus = actionsStore.getRequestStatus(
  "ADD_ACTION",
  props.component.def.id,
  props.actionPrototypeView.id,
);
const removeRequestStatus = actionsStore.getRequestStatus(
  "CANCEL",
  computed(() => props.actionPrototypeView.actionId),
  // ^ this won't accept [] which doesnt bode well
);
</script>
