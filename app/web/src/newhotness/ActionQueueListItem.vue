<template>
  <div
    :class="
      clsx(
        'flex flex-row items-center gap-xs text-sm relative p-2xs min-w-0 w-full border rounded',
        {
          Failed: themeClasses(
            'border-destructive-500 bg-destructive-50',
            'border-destructive-400 bg-[#423131]',
          ),
          Dispatched: '',
          OnHold: '',
          Queued: '',
          Running: '',
        }[action.state],
      )
    "
  >
    <Icon
      v-if="actionRunning"
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
    <Icon
      v-else-if="actionFailed"
      name="restart"
      size="sm"
      @click.stop="retry"
    />
    <Icon
      :class="actionIconClass(action.kind)"
      :name="actionIcon(action.kind)"
      size="sm"
    />

    <div class="flex flex-col flex-grow min-w-0">
      <TruncateWithTooltip class="w-full">
        <span
          :class="
            clsx(
              'text-xs',
              !noInteraction &&
                themeClasses(
                  'group-hover/actioncard:text-action-500',
                  'group-hover/actioncard:text-action-300',
                ),
            )
          "
        >
          <template v-if="action.componentId">
            {{ action.componentSchemaName }}
            {{ action.componentName ?? "unknown" }}
            {{ action.description }}
          </template>
        </span>
      </TruncateWithTooltip>
    </div>

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
        @select="navigateToActionDetails"
      />

      <!-- Go to component -->
      <DropdownMenuItem
        v-if="props.action.componentId"
        icon="component"
        label="Go to component"
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
      <hr class="border-neutral-600 my-xs" />
      <h5 class="text-neutral-400 pl-2xs">APPLY BEFORE:</h5>
      <ol v-if="myDependencies.length > 0">
        <li
          v-for="a in myDependencies"
          :key="a.id"
          class="flex flex-row items-center px-2xs gap-xs"
        >
          <Icon
            :class="actionIconClass(a.kind)"
            :name="actionIcon(a.kind)"
            size="sm"
          />
          <span class="align-baseline leading-[30px]"
            ><strong>{{ actionKindToAbbreviation(a.kind) }}:</strong>
            {{ a.componentSchemaName }}
            {{ a.componentName ?? "unknown" }}
          </span>
        </li>
      </ol>
      <p v-else class="ml-xs">None</p>
      <h5 class="text-neutral-400 pl-2xs">WAITING ON:</h5>
      <ol v-if="dependentOn.length > 0">
        <li
          v-for="a in dependentOn"
          :key="a.id"
          class="flex flex-row items-center px-2xs gap-xs"
        >
          <Icon
            :class="actionIconClass(a.kind)"
            :name="actionIcon(a.kind)"
            size="sm"
          />
          <span class="align-baseline leading-[30px]"
            ><strong>{{ actionKindToAbbreviation(a.kind) }}:</strong>
            {{ a.componentSchemaName }}
            {{ a.componentName ?? "unknown" }}
          </span>
        </li>
      </ol>
      <p v-else class="ml-xs">None</p>
    </DropdownMenu>
    <DetailsPanelMenuIcon
      v-if="!noInteraction"
      :selected="contextMenuRef?.isOpen"
      @click.stop="(e: MouseEvent) => contextMenuRef?.open(e, false)"
    />
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
} from "@si/vue-lib/design-system";
import { ActionKind, ActionState } from "@/api/sdf/dal/action";
import ConfirmHoldModal from "./ConfirmHoldModal.vue";
import DetailsPanelMenuIcon from "./layout_components/DetailsPanelMenuIcon.vue";
import ActionCardLayout from "./ActionCardLayout.vue";
import {
  actionKindToAbbreviation,
  actionIconClass,
  actionIcon,
} from "./logic_composables/action";
import { ActionProposedView } from "./types";
import { routes, useApi } from "./api_composables";

const props = defineProps<{
  action: ActionProposedView;
  actionsById?: Map<string, ActionProposedView>;
  slim?: boolean;
  selected?: boolean;
  failed?: boolean;
  noInteraction?: boolean;
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

// Handle click on the card
const handleClick = () => {
  if (props.noInteraction || !shouldAllowClick.value) {
    return;
  }

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

// Hydrate action IDs into full action objects
const hydrateActions = (
  actionIds: string[] | undefined,
): ActionProposedView[] => {
  if (!actionIds || !props.actionsById) return [];

  const actions: ActionProposedView[] = [];
  for (const id of actionIds) {
    const action = props.actionsById.get(id);
    if (action) {
      actions.push(action);
    }
  }
  return actions;
};

const dependentOn = computed(() => {
  return hydrateActions(props.action.dependentOn);
});

const myDependencies = computed(() => {
  return hydrateActions(props.action.myDependencies);
});

const actionRunning = computed(() => {
  return props.action.state === ActionState.Dispatched;
});

const shouldAllowClick = computed(() => {
  // Allow clicking on failed or running actions
  if (actionFailed.value || actionRunning.value) {
    return true;
  }

  // Allow clicking on on-hold actions that have a funcRunId (indicating previous execution)
  if (actionOnHold.value && props.action.funcRunId) {
    return true;
  }

  return false;
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
</script>

<!-- <template>
  <div
    :class="
      clsx(
        'flex flex-row items-center text-sm relative p-2xs min-w-0 w-full border',
        actionQueueList ? [
          'rounded',
          state && {
            Failed: themeClasses('border-destructive-500', 'border-destructive-400 bg-[#423131]'),
            Dispatched: '',
            OnHold: '',
            Queued: '',
            Running: '',
          }[state],
        ] : [
          'border-transparent',
          !noInteraction && 'cursor-pointer hover:border-action-500 dark:hover:border-action-300 group/actioncard',
          // Background color for selected state
          selected ? 'dark:bg-action-900 bg-action-100' : '',
          // Border color priority: red for highlighted failed, blue for selected, default for others
          highlightedFailed
            ? 'border-destructive-500 dark:border-destructive-400'
            : selected
            ? 'border-action-500 dark:border-action-300'
            : 'dark:border-neutral-800',
        ],
        actionFailed ? 'action-failed' : '',
      )
    "
  >
    <slot name="icons"> </slot>

    <div class="flex flex-col flex-grow min-w-0">
      <TruncateWithTooltip class="w-full">
        <span class="font-bold"> {{ abbr }}: </span>
        <span
          :class="
            clsx(
              'text-xs',
              themeClasses('text-neutral-700', 'text-neutral-200'),
              !noInteraction &&
                themeClasses(
                  'group-hover/actioncard:text-action-500',
                  'group-hover/actioncard:text-action-300',
                ),
            )
          "
        >
          <template v-if="componentId">
            {{ componentSchemaName }}
            {{ componentName ?? "unknown" }}
            {{ description }}
          </template>
        </span>
      </TruncateWithTooltip>
      <div v-if="actor" class="text-neutral-500 dark:text-neutral-400 truncate">
        <span class="font-bold">By:</span> {{ actor }}
      </div>
    </div>

    <slot name="interaction"> </slot>
  </div>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { themeClasses, TruncateWithTooltip } from "@si/vue-lib/design-system";
import { ActionState } from "@/api/sdf/dal/action";

defineProps<{
  noInteraction?: boolean;
  actionQueueList?: boolean;
  selected?: boolean;
  actionFailed: boolean;
  highlightedFailed?: boolean;
  componentId: string | null | undefined;
  componentSchemaName?: string;
  componentName?: string;
  description?: string;
  actor?: string;
  state?: ActionState;
  abbr: string;
}>();
</script> -->
