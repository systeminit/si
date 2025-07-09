<template>
  <div
    v-if="actionViewList.length === 0"
    :class="
      clsx(
        'flex flex-row items-center justify-center',
        'm-xs p-xs border min-h-[calc(100%-16px)]',
        themeClasses('border-neutral-400', 'border-neutral-600'),
      )
    "
  >
    <EmptyState
      icon="arrow--up"
      text="No actions queued yet"
      secondaryText="An action is a specific operation queued up to change your infrastructure, such as creating, refreshing, updating, or deleting a real-world resource."
      class="max-w-[420px]"
    />
  </div>
  <ul v-else class="actions list">
    <ActionCard
      v-for="action in actionViewList"
      :key="action.id"
      :action="action"
      :selected="false"
      :noInteraction="false"
    />
  </ul>
</template>

<script lang="ts" setup>
import { PropType } from "vue";
import { themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { ActionProposedView } from "@/store/actions.store";
import ActionCard from "./ActionCard.vue";
import EmptyState from "./EmptyState.vue";

defineProps({
  actionViewList: {
    type: Array as PropType<ActionProposedView[]>,
    required: true,
  },
});
</script>
