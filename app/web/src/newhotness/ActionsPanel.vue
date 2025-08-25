<template>
  <EmptyState
    v-if="component.toDelete"
    text="Marked for deletion"
    secondaryText="Can't run actions on a component which has been marked for deletion"
    icon="tools"
    class="p-sm"
  />
  <EmptyState
    v-else-if="actionPrototypeViews.length === 0"
    text="No actions available"
    icon="tools"
    class="p-lg"
  />
  <div v-else class="flex flex-col">
    <div
      class="text-sm text-neutral-700 dark:text-neutral-300 p-xs italic border-b dark:border-neutral-600"
    >
      The changes below will run when you click "Apply Changes".
    </div>
    <ActionWidget
      v-for="actionPrototypeView in actionPrototypeViews"
      :key="actionPrototypeView.id"
      :actionPrototypeView="actionPrototypeView"
      :actionId="actionByPrototype[actionPrototypeView.id]?.id"
      :component="component"
    />
  </div>
</template>

<script lang="ts" setup>
import {
  BifrostComponent,
  ComponentInList,
} from "@/workers/types/entity_kind_types";
import ActionWidget from "./ActionWidget.vue";
import EmptyState from "./EmptyState.vue";
import { useComponentActions } from "./logic_composables/component_actions";

const props = defineProps<{
  component: BifrostComponent | ComponentInList;
}>();

const { actionPrototypeViews, actionByPrototype } = useComponentActions(
  () => props.component,
);
</script>
