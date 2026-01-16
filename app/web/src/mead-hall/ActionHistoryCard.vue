<template>
  <ActionCardLayout
    :noInteraction="noInteraction"
    :selected="selected"
    :actionFailed="false"
    :abbr="actionKindToAbbreviation(props.action.kind)"
    :description="action.kind === ActionKind.Manual ? action.description : ''"
    :componentSchemaName="component?.def.schemaName"
    :componentName="component?.def.displayName"
    :componentId="component?.def.id"
    :actor="action.actor"
  >
    <template #icons>
      <Icon :class="resultIconClass(props.action.result)" :name="resultIcon(props.action.result)" size="sm" />
      <Icon :class="actionIconClass(props.action.kind)" :name="actionIcon(props.action.kind)" size="sm" />
    </template>
    <template #interaction>
      <FuncRunTabDropdown
        v-if="!props.noInteraction"
        :funcRunId="props.action.funcRunId"
        @menuClick="(id, slug) => emit('history', id, slug)"
      />
    </template>
  </ActionCardLayout>
</template>

<script lang="ts" setup>
import { computed } from "vue";

import { Icon } from "@si/vue-lib/design-system";
import { useComponentsStore } from "@/store/components.store";
import { ActionKind, ActionId } from "@/api/sdf/dal/action";
import {
  ActionHistoryView,
  actionKindToAbbreviation,
  actionIconClass,
  actionIcon,
  resultIconClass,
  resultIcon,
} from "@/store/actions.store";
import FuncRunTabDropdown from "@/components/FuncRunTabDropdown.vue";
import ActionCardLayout from "./ActionCardLayout.vue";

const componentsStore = useComponentsStore();

const props = defineProps<{
  action: ActionHistoryView;
  slim?: boolean;
  selected?: boolean;
  noInteraction?: boolean;
}>();

// DOUBLE CHECK, this might come over the wire
// if it doesn't delete componentName from the type
const component = computed(() => {
  if (!props.action.componentId) return undefined;
  return componentsStore.allComponentsById[props.action.componentId];
});

const emit = defineEmits<{
  (e: "add"): void;
  (e: "remove"): void;
  (e: "openMenu", mouse: MouseEvent): void;
  (e: "history", id: ActionId, tabSlug: string): void;
}>();
</script>
