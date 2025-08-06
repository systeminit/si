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
      :ref="(el) => setActionCardRef(action.id, el)"
      :action="action"
      :selected="highlightedActionIds.has(action.id)"
      :failed="highlightedActionIds.has(action.id)"
      :noInteraction="false"
    />
  </ul>
</template>

<script lang="ts" setup>
import { PropType, ref, watch, nextTick } from "vue";
import { themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { ActionProposedView } from "./types";
import ActionCard from "./ActionCard.vue";
import EmptyState from "./EmptyState.vue";

const props = defineProps({
  actionViewList: {
    type: Array as PropType<ActionProposedView[]>,
    required: true,
  },
  highlightedActionIds: {
    type: Object as PropType<Set<string>>,
    default: () => new Set(),
  },
});

// Track refs to ActionCard components by action ID
const actionCardRefs = ref<Map<string, InstanceType<typeof ActionCard>>>(
  new Map(),
);

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const setActionCardRef = (actionId: string, el: any) => {
  if (el) {
    actionCardRefs.value.set(actionId, el);
  } else {
    actionCardRefs.value.delete(actionId);
  }
};

// Watch for changes in highlighted actions and scroll to show as many as possible
watch(
  () => props.highlightedActionIds,
  async (newHighlightedIds) => {
    if (newHighlightedIds.size === 0) return;

    // Wait for DOM to update
    await nextTick();

    // Find all highlighted actions and their positions
    const highlightedActions = props.actionViewList.filter((action) =>
      newHighlightedIds.has(action.id),
    );

    if (highlightedActions.length === 0) return;

    if (highlightedActions.length === 1) {
      // Single action: scroll to center it
      const firstAction = highlightedActions[0];
      if (firstAction) {
        const actionCardRef = actionCardRefs.value.get(firstAction.id);
        if (actionCardRef && actionCardRef.$el) {
          actionCardRef.$el.scrollIntoView({
            behavior: "smooth",
            block: "center",
            inline: "nearest",
          });
        }
      }
    } else {
      // Multiple actions: try to show all or scroll to the middle one
      const firstAction = highlightedActions[0];
      const lastAction = highlightedActions[highlightedActions.length - 1];

      if (firstAction && lastAction) {
        const firstActionRef = actionCardRefs.value.get(firstAction.id);
        const lastActionRef = actionCardRefs.value.get(lastAction.id);

        if (firstActionRef?.$el && lastActionRef?.$el) {
          // Get the container element (scrollable parent)
          const container = firstActionRef.$el.closest(".scrollable");
          if (container) {
            const firstRect = firstActionRef.$el.getBoundingClientRect();
            const lastRect = lastActionRef.$el.getBoundingClientRect();
            const containerRect = container.getBoundingClientRect();

            const totalHeight = lastRect.bottom - firstRect.top;
            const containerHeight = containerRect.height;

            if (totalHeight <= containerHeight) {
              // All actions can fit in view, scroll to show them all
              const middleY = (firstRect.top + lastRect.bottom) / 2;
              const containerMiddleY =
                containerRect.top + containerRect.height / 2;
              const scrollOffset = middleY - containerMiddleY;

              container.scrollBy({
                top: scrollOffset,
                behavior: "smooth",
              });
            } else {
              // Actions don't all fit, scroll to the middle action
              const middleIndex = Math.floor(highlightedActions.length / 2);
              const middleAction = highlightedActions[middleIndex];
              if (middleAction) {
                const middleActionRef = actionCardRefs.value.get(
                  middleAction.id,
                );
                if (middleActionRef?.$el) {
                  middleActionRef.$el.scrollIntoView({
                    behavior: "smooth",
                    block: "center",
                    inline: "nearest",
                  });
                }
              }
            }
          }
        }
      }
    }
  },
  { deep: true, flush: "post" },
);
</script>
