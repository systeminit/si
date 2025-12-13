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
  <div v-else class="actions list">
    <template v-for="section in actionDisplayLists" :key="section.title">
      <div
        v-if="section.actions.length > 0"
        class="flex flex-col items-stretch gap-xs p-xs"
      >
        <div
          :class="
            clsx(
              'flex flex-row items-center gap-xs w-full h-8',
              themeClasses(
                'text-neutral-600 [&_*]:border-neutral-400',
                'text-neutral-400 [&_*]:border-neutral-600',
              ),
            )
          "
        >
          <div class="flex-none">
            {{ section.title }}
          </div>
          <div class="border-b flex-1 h-0" />
          <NewButton
            v-if="section.title === 'Failed'"
            label="Retry All"
            icon="restart"
            @click="retryAll"
          />
        </div>
        <ActionQueueListItem
          v-for="action in section.actions"
          :key="action.id"
          :ref="(el) => setActionQueueListItemRef(action.id, el)"
          :action="action"
          :actionsById="actionsById"
          :noInteraction="false"
        />
      </div>
    </template>
  </div>
</template>

<script lang="ts" setup>
import { PropType, ref, computed, watch, nextTick } from "vue";
import { NewButton, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { ActionState } from "@/api/sdf/dal/action";
import { ActionProposedView } from "./types";
import EmptyState from "./EmptyState.vue";
import ActionQueueListItem from "./ActionQueueListItem.vue";
import { routes, useApi } from "./api_composables";

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

// Check if an action has a parent with the state "Queued" or "OnHold"
const hasDisplayedParent = (child: ActionProposedView) => {
  for (const parentId of child.dependentOn) {
    const parent = actionsById.value.get(parentId);
    if (
      parent &&
      (parent.state === ActionState.Queued ||
        parent.state === ActionState.OnHold)
    ) {
      return true;
    }
  }
  return false;
};

// Create a map of actions by ID for looking up dependencies
const actionsById = computed(() => {
  const map = new Map<string, ActionProposedView>();
  for (const action of props.actionViewList) {
    map.set(action.id, action);
  }
  return map;
});

const queuedShouldShow = (action: ActionProposedView) => {
  const blockedByParent = (action.holdStatusInfluencedBy?.length ?? 0) > 0;
  const hasQueuedOrOnHoldParent = !!action.holdStatusInfluencedBy.find(
    (parentId) =>
      actionsById.value.get(parentId) &&
      (actionsById.value.get(parentId)?.state === ActionState.Queued ||
        actionsById.value.get(parentId)?.state === ActionState.OnHold),
  );

  if (blockedByParent && hasQueuedOrOnHoldParent) {
    return false;
  }
  return true;
};

// Create sorted lists of actions - Failed, Running, Queued
const actionDisplayLists = computed(() => [
  {
    title: "Failed",
    actions: props.actionViewList.filter(
      (action) => action.state === ActionState.Failed,
    ) as ActionProposedView[],
  },
  {
    title: "Running",
    actions: props.actionViewList.filter(
      (action) =>
        action.state === ActionState.Running ||
        action.state === ActionState.Dispatched,
    ) as ActionProposedView[],
  },
  {
    title: "Queued",
    actions: props.actionViewList.filter(
      (action) =>
        action.state === ActionState.Queued &&
        !hasDisplayedParent(action) &&
        queuedShouldShow(action),
    ) as ActionProposedView[],
  },
  {
    title: "On Hold",
    actions: props.actionViewList.filter(
      (action) =>
        action.state === ActionState.OnHold && !hasDisplayedParent(action),
    ) as ActionProposedView[],
  },
]);

const retryApi = useApi();
const retryAll = () => {
  const failedActions = actionDisplayLists.value[0]?.actions;
  if (failedActions && failedActions.length > 0) {
    failedActions.forEach((action) => {
      const call = retryApi.endpoint(routes.ActionRetry, { id: action.id });
      call.put({});
    });
  }
};

// Track refs to ActionCard components by action ID
const actionQueueListItemRefs = ref<
  Map<string, InstanceType<typeof ActionQueueListItem>>
>(new Map());

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const setActionQueueListItemRef = (actionId: string, el: any) => {
  if (el) {
    actionQueueListItemRefs.value.set(actionId, el);
  } else {
    actionQueueListItemRefs.value.delete(actionId);
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
        const actionCardRef = actionQueueListItemRefs.value.get(firstAction.id);
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
        const firstActionRef = actionQueueListItemRefs.value.get(
          firstAction.id,
        );
        const lastActionRef = actionQueueListItemRefs.value.get(lastAction.id);

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
                const middleActionRef = actionQueueListItemRefs.value.get(
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
