<template>
  <div
    class="cursor-pointer"
    :class="
      clsx(
        'flex flex-row items-center gap-xs p-2xs border-x border-b',
        themeClasses('border-neutral-400', 'border-neutral-600'),
      )
    "
    @click="clickHandler"
  >
    <StatusIndicatorIcon type="action" :status="actionPrototypeView.kind" tone="inherit" class="flex-none" />
    <div class="font-bold leading-normal text-sm">
      {{ actionPrototypeView.displayName || actionPrototypeView.name }}
    </div>

    <Toggle
      :selected="!!props.actionId"
      class="flex-none ml-auto"
      onLabel="Queued"
      offLabel="UnQueued"
      labelWidth="w-[70px]"
    />
    <Icon v-if="actionBifrosting" name="loader" size="sm" />
  </div>
</template>

<script setup lang="ts">
import * as _ from "lodash-es";
import clsx from "clsx";
import { Icon, themeClasses, Toggle } from "@si/vue-lib/design-system";
import StatusIndicatorIcon from "@/components/StatusIndicatorIcon.vue";
import { ActionId } from "@/api/sdf/dal/action";
import { ActionPrototypeView, BifrostComponent, ComponentInList } from "@/workers/types/entity_kind_types";
import { useComponentActions } from "./logic_composables/component_actions";

const props = defineProps<{
  component: BifrostComponent | ComponentInList;
  actionPrototypeView: ActionPrototypeView;
  actionId?: ActionId;
}>();

const { toggleActionHandler } = useComponentActions(() => props.component);

const { handleToggle, bifrosting: actionBifrosting } = toggleActionHandler(
  props.actionPrototypeView,
  () => props.actionId,
);

const clickHandler = handleToggle;
</script>
