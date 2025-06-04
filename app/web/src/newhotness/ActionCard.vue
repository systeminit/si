<template>
  <ActionCardLayout
    :noInteraction="noInteraction"
    :selected="selected"
    :actionFailed="actionFailed"
    :abbr="actionKindToAbbreviation(action.kind)"
    :description="action.kind === ActionKind.Manual ? action.description : ''"
    :componentSchemaName="action.componentSchemaName"
    :componentName="action.componentName"
    :componentId="action.componentId"
    :actor="action.actor"
    @click="handleClick"
  >
    <template #icons>
      <Icon
        v-if="actionQueued"
        :class="
          clsx(
            themeClasses('text-neutral-600', 'text-neutral-300'),
            'translate-y-[-2px]',
          )
        "
        name="nested-arrow-right"
        size="sm"
      />
      <Icon
        v-else-if="actionRunning"
        :class="clsx(themeClasses('text-action-300', 'text-action-300'))"
        name="loader"
        size="sm"
        class="animate-spin"
      />
      <Icon
        v-else-if="actionOnHold"
        :class="
          clsx(
            action.holdStatusInfluencedBy?.length > 0
              ? [
                  'opacity-30',
                  themeClasses('text-warning-500', 'text-warning-300'),
                ]
              : themeClasses('text-warning-400', 'text-warning-300'),
          )
        "
        name="circle-stop"
        size="sm"
      />
      <template v-else-if="actionFailed">
        <Icon
          :class="clsx(themeClasses('text-action-700', 'text-action-300'))"
          name="play"
          size="sm"
          @click.stop="retry"
        />
        <Icon
          :class="
            clsx(themeClasses('text-destructive-500', 'text-destructive-600'))
          "
          name="x-hex-outline"
          size="sm"
        />
      </template>
      <Icon
        :class="actionIconClass(action.kind)"
        :name="actionIcon(action.kind)"
        size="sm"
      />
    </template>
    <template #interaction>
      <ConfirmHoldModal
        v-if="!noInteraction"
        ref="confirmRef"
        :ok="finishHold"
      />
      <DropdownMenu
        v-if="!noInteraction"
        ref="contextMenuRef"
        :forceAbove="false"
        forceAlignRight
      >
        <h5 class="text-neutral-400 pl-2xs">ACTIONS:</h5>

        <!-- View action details -->
        <DropdownMenuItem
          icon="eye"
          label="View details"
          :disabled="!props.action.funcRunId"
          @select="navigateToActionDetailsProtected"
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
          iconClass="text-action-400"
          label="Put in Queue"
          @select="retry"
        />
        <DropdownMenuItem
          icon="x"
          iconClass="text-destructive-500 dark:text-destructive-600"
          label="Remove from list"
          @select="remove"
        />
      </DropdownMenu>
      <DetailsPanelMenuIcon
        v-if="!noInteraction"
        @click.stop="(e) => contextMenuRef?.open(e, false)"
      />
    </template>
  </ActionCardLayout>
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
} from "@si/vue-lib/design-system";
import { ActionKind, ActionState } from "@/api/sdf/dal/action";
import {
  actionKindToAbbreviation,
  actionIconClass,
  actionIcon,
  ActionProposedView,
} from "@/store/actions.store";
import ConfirmHoldModal from "@/components/Actions/ConfirmHoldModal.vue";
import DetailsPanelMenuIcon from "@/newhotness/layout_components/DetailsPanelMenuIcon.vue";
import ActionCardLayout from "@/mead-hall/ActionCardLayout.vue";
import { routes, useApi } from "./api_composables";

const props = defineProps<{
  action: ActionProposedView;
  slim?: boolean;
  selected?: boolean;
  noInteraction?: boolean;
}>();

const emit = defineEmits<{
  (e: "click", action: ActionProposedView): void;
}>();

const router = useRouter();
const route = useRoute();
const confirmRef = ref<InstanceType<typeof ConfirmHoldModal> | null>(null);
const contextMenuRef = ref<InstanceType<typeof DropdownMenu>>();

// Navigate to action details only if the func run ID is populated for the action
const navigateToActionDetailsProtected = () => {
  if (!props.action.funcRunId) return;
  navigateToActionDetails();
};

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

// Note: This function is commented out but kept for future use
// when we might want to navigate directly to function run details
/*
const navigateToFuncRunDetails = () => {
  if (props.action.funcRunId) {
    router.push({
      name: "new-hotness-func-run",
      params: {
        workspacePk: route.params.workspacePk,
        changeSetId: route.params.changeSetId,
        funcRunId: props.action.funcRunId,
      },
    });
  }
};
*/

// Handle click on the card
const handleClick = () => {
  if (!props.action.funcRunId) return;

  emit("click", props.action);

  // Navigate to action details which will show the latest function run
  navigateToActionDetails();
};

const actionOnHold = computed(() => {
  return (
    props.action.state === ActionState.OnHold ||
    (props.action.holdStatusInfluencedBy?.length ?? 0) > 0
  );
});

const actionFailed = computed(() => {
  return props.action.state === ActionState.Failed;
});

const actionRunning = computed(() => {
  return props.action.state === ActionState.Running;
});

const actionQueued = computed(() => {
  return props.action.state === ActionState.Queued;
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
  await call.put({});
  confirmRef.value?.close();
};

const retryApi = useApi();
const retry = async () => {
  const call = retryApi.endpoint(routes.ActionRetry, { id: props.action.id });
  await call.put({});
};

const removeApi = useApi();
const remove = async () => {
  const call = removeApi.endpoint(routes.ActionCancel, { id: props.action.id });
  await call.put({});
};
</script>
