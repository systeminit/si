<template>
  <div
    :class="
      clsx(
        'flex flex-row items-center text-sm relative p-2xs min-w-0 w-full border border-transparent',
        !props.noInteraction
          ? 'cursor-pointer hover:border-action-500 dark:hover:border-action-300 group/actioncard'
          : '',
        selected
          ? 'dark:bg-action-900 bg-action-100 border-action-500 dark:border-action-300'
          : 'dark:border-neutral-800',
        actionFailed ? 'action-failed' : '',
      )
    "
  >
    <template v-if="actionProposed">
      <Icon
        v-if="actionQueued"
        :class="clsx(themeClasses('text-neutral-600', 'text-neutral-300'), 'translate-y-[-2px]')"
        name="nested-arrow-right"
        size="sm"
      />
      <Icon
        v-else-if="actionRunning"
        :class="clsx(themeClasses('text-action-300', 'text-action-300'))"
        name="loader"
        size="sm"
      />
      <Icon
        v-else-if="actionOnHold"
        :class="
          clsx(
            holdStatusInfluencedBy.length > 0
              ? ['opacity-30', themeClasses('text-warning-500', 'text-warning-300')]
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
          :class="clsx(themeClasses('text-destructive-500', 'text-destructive-600'))"
          name="x-hex-outline"
          size="sm"
        />
      </template>
    </template>
    <template v-else-if="actionHistory">
      <Icon :class="resultIconClass" :name="resultIcon" size="sm" />
    </template>
    <Icon :class="actionIconClass(props.action.kind)" :name="actionIcon(props.action.kind)" size="sm" />
    <div class="flex flex-col flex-grow min-w-0">
      <TruncateWithTooltip class="w-full">
        <span class="font-bold"> {{ actionKindToAbbreviation(props.action.kind) }}: </span>
        <span
          :class="
            clsx(
              themeClasses('text-neutral-700', 'text-neutral-200'),
              !noInteraction &&
                themeClasses('group-hover/actioncard:text-action-500', 'group-hover/actioncard:text-action-300'),
            )
          "
        >
          <template v-if="component">
            {{ component?.def.schemaName }}
            {{ component?.def.displayName ?? "unknown" }}
            {{ props.action.kind === ActionKind.Manual ? props.action.description : "" }}
          </template>
          <template v-else-if="actionHistory">
            {{ actionHistory.schemaName }}
            {{ actionHistory.componentName }}
          </template>
        </span>
      </TruncateWithTooltip>
      <div v-if="props.action.actor" class="text-neutral-500 dark:text-neutral-400 truncate">
        <span class="font-bold">By:</span> {{ props.action.actor }}
      </div>
    </div>
    <ConfirmHoldModal v-if="!props.noInteraction" ref="confirmRef" :ok="finishHold" />
    <DropdownMenu
      v-if="!props.noInteraction && actionProposed"
      ref="contextMenuRef"
      :forceAbove="false"
      forceAlignRight
    >
      <h5 class="text-neutral-400 pl-2xs">ACTIONS:</h5>
      <DropdownMenuItem
        v-if="actionProposed.state === ActionState.Queued"
        :onSelect="hold"
        icon="circle-stop"
        iconClass="text-warning-400"
        label="Put on hold"
      />
      <DropdownMenuItem
        v-if="actionProposed.state === ActionState.OnHold"
        :onSelect="retry"
        icon="nested-arrow-right"
        iconClass="text-action-400"
        label="Put in Queue"
      />
      <DropdownMenuItem
        :onSelect="remove"
        icon="x"
        iconClass="text-destructive-500 dark:text-destructive-600"
        label="Remove from list"
      />
      <hr class="border-neutral-600 my-xs" />
      <h5 class="text-neutral-400 pl-2xs">APPLY BEFORE:</h5>
      <ol v-if="myDependencies.length > 0">
        <li v-for="a in myDependencies" :key="a.id" class="flex flex-row items-center px-2xs gap-xs">
          <Icon :class="actionIconClass(a.kind)" :name="actionIcon(a.kind)" size="sm" />
          <span class="align-baseline leading-[30px]"
            ><strong>{{ actionKindToAbbreviation(a.kind) }}:</strong>
            {{ a.component?.def.schemaName }}
            {{ a.component?.def.displayName ?? "unknown" }}
          </span>
        </li>
      </ol>
      <p v-else class="ml-xs">None</p>
      <h5 class="text-neutral-400 pl-2xs">WAITING ON:</h5>
      <ol v-if="dependentOn.length > 0">
        <li v-for="a in dependentOn" :key="a.id" class="flex flex-row items-center px-2xs gap-xs">
          <Icon :class="actionIconClass(a.kind)" :name="actionIcon(a.kind)" size="sm" />
          <span class="align-baseline leading-[30px]"
            ><strong>{{ actionKindToAbbreviation(a.kind) }}:</strong>
            {{ a.component?.def.schemaName }}
            {{ a.component?.def.displayName ?? "unknown" }}
          </span>
        </li>
      </ol>
      <p v-else class="ml-xs">None</p>
    </DropdownMenu>
    <DetailsPanelMenuIcon
      v-if="!props.noInteraction"
      @click="
        (e: MouseEvent) => {
          contextMenuRef?.open(e, false);
        }
      "
    />
    <FuncRunTabDropdown
      v-if="!props.noInteraction && actionHistory"
      :funcRunId="actionHistory.funcRunId"
      @menuClick="(id, slug) => emit('history', id, slug)"
    />
  </div>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";

import {
  Icon,
  IconNames,
  themeClasses,
  DropdownMenu,
  DropdownMenuItem,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { useComponentsStore } from "@/store/components.store";
import { ActionKind, ActionState, ActionId } from "@/api/sdf/dal/action";
import { ActionView, useActionsStore, ActionProposedView, ActionHistoryView } from "@/store/actions.store";
import ConfirmHoldModal from "./ConfirmHoldModal.vue";
import FuncRunTabDropdown from "../FuncRunTabDropdown.vue";

import { DiagramGroupData, DiagramNodeData } from "../ModelingDiagram/diagram_types";
import DetailsPanelMenuIcon from "../DetailsPanelMenuIcon.vue";

const componentsStore = useComponentsStore();
const actionStore = useActionsStore();

const props = defineProps<{
  action: ActionView;
  slim?: boolean;
  selected?: boolean;
  noInteraction?: boolean;
}>();

// This will populate with an ActionProposedView if the ActionView passed in has state
const actionProposed = computed(() => {
  if ("state" in props.action) {
    return props.action as ActionProposedView;
  } else {
    return undefined;
  }
});

// This will populate with an ActionHistoryView if the ActionView passed in has result
const actionHistory = computed(() => {
  if ("result" in props.action) {
    return props.action as ActionHistoryView;
  } else {
    return undefined;
  }
});

const confirmRef = ref<InstanceType<typeof ConfirmHoldModal> | null>(null);

const hold = () => {
  if (actionProposed.value) {
    const l = actionProposed.value.myDependencies?.length;
    if (l && l > 0) confirmRef.value?.open();
    else finishHold();
  } else return undefined;
};

const finishHold = () => {
  actionStore.PUT_ACTION_ON_HOLD([props.action.id]);
  confirmRef.value?.close();
};

const remove = () => {
  actionStore.CANCEL([props.action.id]);
};

const retry = () => {
  actionStore.RETRY([props.action.id]);
};

const contextMenuRef = ref<InstanceType<typeof DropdownMenu>>();

const actionOnHold = computed(() => {
  if (actionProposed.value && "state" in actionProposed.value)
    return actionProposed.value.state === ActionState.OnHold || actionProposed.value.holdStatusInfluencedBy?.length > 0;
  else return false;
});
const actionFailed = computed(() => {
  if (actionProposed.value) return actionProposed.value.state === ActionState.Failed;
  else return false;
});
const actionRunning = computed(() => {
  if (actionProposed.value) return actionProposed.value.state === ActionState.Running;
  else return false;
});
const actionQueued = computed(() => {
  if (actionProposed.value) return actionProposed.value.state === ActionState.Queued;
  else return false;
});

type ActionViewWithComponent = ActionView & {
  component: DiagramGroupData | DiagramNodeData | undefined;
};

const hydrateActions = (actionList: ActionId[] | undefined) => {
  const actions = [] as ActionViewWithComponent[];
  if (actionList) {
    for (const id of actionList) {
      const _a = actionStore.actionsById.get(id);
      const a = _a as unknown as ActionViewWithComponent;
      if (a) {
        if (a.componentId) {
          a.component = componentsStore.allComponentsById[a.componentId];
        }
        actions.push(a);
      }
    }
  }
  return actions;
};

const dependentOn = computed(() => {
  if (actionProposed.value) {
    return hydrateActions(actionProposed.value.dependentOn);
  } else return [];
});
const myDependencies = computed(() => {
  if (actionProposed.value) {
    return hydrateActions(actionProposed.value.myDependencies);
  } else return [];
});
const holdStatusInfluencedBy = computed(() => {
  if (actionProposed.value) {
    return hydrateActions(actionProposed.value.holdStatusInfluencedBy);
  } else return [];
});

const resultIconClass = computed(() => {
  if (actionHistory.value) {
    return {
      Success: "text-success-600",
      Failure: "text-destructive-500 dark:text-destructive-600",
      Unknown: "text-warning-600",
    }[actionHistory.value.result];
  } else return undefined;
});

const resultIcon = computed(() => {
  if (actionHistory.value) {
    const p = {
      // outlined icons represent status about the simulation
      // filled icons represent status about resources in the real world
      // so we used filled in icons here
      Success: "check-hex",
      Failure: "x-hex",
      Unknown: "question-hex-outline", // TODO, get a non-outlined icon here
    }[actionHistory.value.result] as IconNames;
    return p;
  } else return "none" as IconNames;
});

const actionIconClass = (kind: ActionKind) => {
  return {
    Create: "text-success-600",
    Destroy: "text-destructive-500 dark:text-destructive-600",
    Refresh: "text-action-600",
    Manual: "text-action-600",
    Update: "text-warning-600",
  }[kind];
};

const actionIcon = (kind: ActionKind) => {
  return {
    Create: "plus",
    Destroy: "trash",
    Refresh: "refresh",
    Manual: "play",
    Update: "tilde",
  }[kind] as IconNames;
};

const actionKindToAbbreviation = (actionKind: ActionKind) => {
  return {
    Create: "CRT",
    Destroy: "DLT",
    Refresh: "RFH",
    Manual: "MNL",
    Update: "UPT",
  }[actionKind];
};

const component = computed(() => {
  if (!props.action.componentId) return undefined;
  const component = componentsStore.allComponentsById[props.action.componentId];
  if (actionHistory.value && !component) return undefined;
  return component;
});

const emit = defineEmits<{
  (e: "add"): void;
  (e: "remove"): void;
  (e: "openMenu", mouse: MouseEvent): void;
  (e: "history", id: ActionId, tabSlug: string): void;
}>();
</script>

<style lang="less">
@keyframes flashRed {
  from {
    background-color: transparent;
  }
  50% {
    background-color: #dc2626; //bg-destructive-600
  }
  to {
    background-color: transparent;
  }
}
.action-failed {
  animation: 0.75s ease-in flashRed;
}
</style>
