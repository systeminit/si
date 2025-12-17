<template>
  <div
    :class="
      clsx(
        'flex flex-col gap-2xs items-stretch relative min-w-0 w-full',
        !child && 'border rounded p-2xs',
        {
          Failed: themeClasses(
            'border-destructive-500 bg-destructive-50',
            'border-destructive-400 bg-newhotness-destructive2',
          ),
          OnHold: themeClasses(
            'border-warning-500 bg-warning-50',
            'border-warning-300 bg-newhotness-warningdark',
          ),
          Queued: themeClasses('border-neutral-400', 'border-neutral-600'),
          Dispatched: runningClasses,
          Running: runningClasses,
        }[action.state],
      )
    "
  >
    <div class="flex flex-row items-center gap-xs">
      <Icon
        v-if="actionRunning"
        :class="clsx(themeClasses('text-action-300', 'text-action-300'))"
        name="loader"
        size="sm"
        class="animate-spin flex-none"
      />
      <Icon
        v-else-if="action.state === ActionState.OnHold"
        :class="
          clsx(
            'flex-none',
            themeClasses('text-warning-400', 'text-warning-300'),
          )
        "
        name="circle-stop"
        size="sm"
      />
      <NewButton
        v-else-if="actionFailed"
        icon="restart"
        tone="nostyle"
        size="sm"
        tooltip="Retry"
        class="flex-none"
        @click.stop="retry"
      />
      <Icon
        v-else
        :class="clsx(actionIconClass(action.kind), 'flex-none')"
        :name="actionIcon(action.kind)"
        size="sm"
      />

      <TruncateWithTooltip class="flex-grow min-w-0 text-xs py-2xs">
        <template v-if="action.componentId">
          {{ action.componentSchemaName }}
          {{ action.componentName ?? "unknown" }}
          {{ action.description }}
        </template>
        <template v-else>
          {{ action.name }}
          {{ action.description }}
        </template>
      </TruncateWithTooltip>

      <DetailsPanelMenuIcon
        :selected="contextMenuRef?.isOpen"
        @click.stop="(e: MouseEvent) => contextMenuRef?.open(e, false)"
      />
    </div>
    <template
      v-if="
        action.state === ActionState.OnHold ||
        action.state === ActionState.Queued
      "
    >
      <ActionQueueListItem
        v-for="subaction in childActions"
        :key="subaction.id"
        :action="subaction"
        :actionsById="actionsById"
        child
        :actionChildren="props.actionChildren"
        :class="!child && 'pl-md'"
      />
    </template>

    <ConfirmHoldModal ref="confirmRef" :ok="finishHold" />
    <DropdownMenu ref="contextMenuRef" variant="actionmenu" forceAlignRight>
      <!-- View action details -->
      <DropdownMenuItem
        icon="func"
        label="Go to action function"
        @select="navigateToActionDetails"
      />

      <!-- Go to component -->
      <DropdownMenuItem
        v-if="props.action.componentId"
        icon="component"
        label="Go to component details"
        @select="navigateToComponent"
      />

      <!-- Action state controls -->
      <DropdownMenuItem
        v-if="action.state === ActionState.Queued"
        icon="circle-stop"
        iconClass="text-warning-400"
        label="Put on hold"
        @select="hold"
      />
      <DropdownMenuItem
        v-if="action.state === ActionState.OnHold"
        icon="nested-arrow-right"
        :iconClass="themeClasses('text-action-500', 'text-action-300')"
        label="Put in Queue"
        @select="retry"
      />
      <hr class="border-neutral-600" />
      <DropdownMenuItem
        v-if="
          action.state !== ActionState.Running &&
          action.state !== ActionState.Dispatched
        "
        icon="minus-circle-outline"
        destructiveOption
        :label="
          action.state === ActionState.Failed
            ? 'Remove from list'
            : 'Remove from queue'
        "
        @select="remove"
      />
    </DropdownMenu>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import { useRouter, useRoute } from "vue-router";
import clsx from "clsx";
import {
  Icon,
  themeClasses,
  DropdownMenu,
  DropdownMenuItem,
  TruncateWithTooltip,
  NewButton,
} from "@si/vue-lib/design-system";
import { tw } from "@si/vue-lib";
import { ActionState } from "@/api/sdf/dal/action";
import ConfirmHoldModal from "./ConfirmHoldModal.vue";
import DetailsPanelMenuIcon from "./layout_components/DetailsPanelMenuIcon.vue";
import { actionIconClass, actionIcon } from "./logic_composables/action";
import { ActionProposedView } from "./types";
import { routes, useApi } from "./api_composables";

const props = defineProps<{
  action: ActionProposedView;
  actionsById?: Map<string, ActionProposedView>;
  child?: boolean;
  actionChildren: Map<string, ActionProposedView[]>;
}>();

const emit = defineEmits<{
  (e: "click", action: ActionProposedView): void;
}>();

const router = useRouter();
const route = useRoute();
const confirmRef = ref<InstanceType<typeof ConfirmHoldModal> | null>(null);
const contextMenuRef = ref<InstanceType<typeof DropdownMenu>>();

// Navigate to action details
const navigateToActionDetails = () => {
  router.push({
    name: "new-hotness-action",
    params: {
      workspacePk: route.params.workspacePk,
      changeSetId: route.params.changeSetId,
      actionId: props.action.id,
    },
  });
};

// Navigate to component
const navigateToComponent = () => {
  if (!props.action.componentId) return;

  router.push({
    name: "new-hotness-component",
    params: {
      workspacePk: route.params.workspacePk,
      changeSetId: route.params.changeSetId,
      componentId: props.action.componentId,
    },
  });
};

const actionFailed = computed(() => {
  return props.action.state === ActionState.Failed;
});

// Child actions are only used for actions which are Queued or OnHold
// Only include DIRECT children (actions that list THIS action as a parent)
const childActions = computed(() => {
  return [...(props.actionChildren.get(props.action.id) || [])];
});

const actionRunning = computed(() => {
  return (
    props.action.state === ActionState.Dispatched ||
    props.action.state === ActionState.Running
  );
});

// Action handling methods
const hold = () => {
  const hasDependencies = (props.action.myDependencies?.length ?? 0) > 0;
  if (hasDependencies) {
    confirmRef.value?.open();
  } else {
    finishHold();
  }
};

const holdApi = useApi();
const finishHold = async () => {
  const call = holdApi.endpoint(routes.ActionHold, { id: props.action.id });

  // This route can mutate head, so we do not need to handle new change set semantics.
  await call.put({});
  confirmRef.value?.close();
};

const retryApi = useApi();
const retry = async () => {
  const call = retryApi.endpoint(routes.ActionRetry, { id: props.action.id });

  // This route can mutate head, so we do not need to handle new change set semantics.
  await call.put({});
};

const removeApi = useApi();
const remove = async () => {
  const call = removeApi.endpoint(routes.ActionCancel, { id: props.action.id });

  // This route can mutate head, so we do not need to handle new change set semantics.
  await call.put({});
};

const runningClasses = computed(() =>
  themeClasses(
    tw`border-action-500 bg-action-50`,
    tw`border-action-300 bg-neutral-700`,
  ),
);
</script>
