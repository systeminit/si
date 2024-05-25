<template>
  <div
    :class="
      clsx(
        'flex flex-row items-center text-sm relative p-2xs min-w-0 w-full border border-transparent',
        !props.noInteraction
          ? 'cursor-pointer hover:border-action-500 dark:hover:border-action-300'
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
        name="nested-arrow-right"
        :class="clsx(themeClasses('text-neutral-600', 'text-neutral-300'))"
      />
      <Icon
        v-else-if="actionRunning"
        name="loader"
        :class="clsx(themeClasses('text-action-300', 'text-action-300'))"
      />
      <Icon
        v-else-if="actionOnHold"
        name="circle-stop"
        :class="
          clsx(
            holdStatusInfluencedBy.length > 0
              ? [
                  'opacity-30',
                  themeClasses('text-warning-500', 'text-warning-300'),
                ]
              : themeClasses('text-warning-400', 'text-warning-300'),
          )
        "
        size="lg"
      />
      <template v-else-if="actionFailed">
        <Icon
          name="play"
          :class="clsx(themeClasses('text-action-700', 'text-action-300'))"
          @click="retry"
        />
        <Icon
          name="x-hex-outline"
          :class="
            clsx(themeClasses('text-destructive-700', 'text-destructive-700'))
          "
        />
      </template>
    </template>
    <template v-else-if="actionHistory">
      <!-- TODO(Wendy) - need to implement a way to retry failed actions here -->
      <Icon size="lg" :name="resultIcon" :class="resultIconClass" />
    </template>
    <Icon
      :name="actionIcon(props.action.kind)"
      :class="actionIconClass(props.action.kind)"
      size="lg"
    />
    <div class="flex flex-col flex-grow">
      <div>
        <span class="font-bold">
          {{ actionKindToAbbreviation(props.action.kind) }}:
        </span>
        <span
          :class="
            clsx(
              props.noInteraction
                ? 'text-neutral-500 dark:text-neutral-400'
                : 'truncate cursor-pointer ',
              component?.displayName && !props.noInteraction
                ? 'dark:text-action-300 text-action-500'
                : 'text-neutral-500 dark:text-neutral-400',
              isHover && !props.noInteraction && 'underline',
            )
          "
          @click="onClick"
          @mouseenter="onHoverStart"
          @mouseleave="onHoverEnd"
        >
          {{ component?.displayName ?? "unknown" }}
        </span>
      </div>
      <div
        v-if="action.actor"
        class="text-neutral-500 dark:text-neutral-400 truncate"
      >
        <span class="font-bold">By:</span> {{ action.actor }}
      </div>
    </div>
    <ConfirmHoldModal
      v-if="!props.noInteraction"
      ref="confirmRef"
      :ok="finishHold"
    />
    <DropdownMenu
      v-if="!props.noInteraction && actionProposed"
      ref="contextMenuRef"
      :forceAbove="false"
      forceAlignRight
    >
      <h5 class="dark:text-neutral-400 text-neutral-700">ACTIONS:</h5>
      <DropdownMenuItem
        v-if="actionProposed.state === ActionState.Queued"
        label="Put on hold"
        icon="circle-stop"
        iconClass="text-warning-400"
        :onSelect="hold"
      />
      <DropdownMenuItem
        v-if="actionProposed.state === ActionState.OnHold"
        label="Put in Queue"
        icon="nested-arrow-right"
        iconClass="text-action-400"
        :onSelect="retry"
      />
      <DropdownMenuItem
        label="Remove from list"
        icon="x"
        iconClass="text-destructive-400"
        :onSelect="remove"
      />
      <hr class="border-neutral-600 my-xs" />
      <h5 class="dark:text-neutral-400 text-neutral-700">APPLY BEFORE:</h5>
      <ol v-if="myDependencies.length > 0">
        <li v-for="a in myDependencies" :key="a.id" class="flex flex-row">
          <Icon
            :name="actionIcon(a.kind)"
            :class="actionIconClass(a.kind)"
            size="lg"
          />
          <span class="align-baseline leading-[30px]"
            ><strong>{{ actionKindToAbbreviation(a.kind) }}:</strong>
            {{ a.component?.displayName ?? "unknown" }}</span
          >
        </li>
      </ol>
      <p v-else class="ml-xs">None</p>
      <h5 class="dark:text-neutral-400 text-neutral-700">WAITING ON:</h5>
      <ol v-if="dependentOn.length > 0">
        <li v-for="a in dependentOn" :key="a.id" class="flex flex-row">
          <Icon
            :name="actionIcon(a.kind)"
            :class="actionIconClass(a.kind)"
            size="lg"
          />
          <span class="align-baseline leading-[30px]"
            ><strong>{{ actionKindToAbbreviation(a.kind) }}:</strong>
            {{ a.component?.displayName ?? "unknown" }}</span
          >
        </li>
      </ol>
      <p v-else class="ml-xs">None</p>
    </DropdownMenu>
    <DropdownMenu
      v-else-if="!props.noInteraction && actionHistory"
      ref="contextMenuRef"
      :forceAbove="false"
      forceAlignRight
    >
      <DropdownMenuItem
        label="Resource Result"
        :onSelect="
          () => {
            emit('history', action.id, 'resourceResult');
          }
        "
      />
      <DropdownMenuItem
        label="Code Executed"
        :onSelect="
          () => {
            emit('history', action.id, 'codeExecuted');
          }
        "
      />
      <DropdownMenuItem
        label="Logs"
        :onSelect="
          () => {
            emit('history', action.id, 'logs');
          }
        "
      />
      <DropdownMenuItem
        label="Arguments"
        :onSelect="
          () => {
            emit('history', action.id, 'arguments');
          }
        "
      />
    </DropdownMenu>
    <DetailsPanelMenuIcon
      v-if="!props.noInteraction"
      @click="
        (e) => {
          contextMenuRef?.open(e, false);
        }
      "
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";

import {
  Icon,
  IconNames,
  themeClasses,
  DropdownMenu,
  DropdownMenuItem,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { FullComponent, useComponentsStore } from "@/store/components.store";
import {
  ActionKind,
  ActionState,
  ActionView,
  useActionsStore,
  ActionId,
  ActionProposedView,
  ActionHistoryView,
} from "@/store/actions.store";
import ConfirmHoldModal from "./ConfirmHoldModal.vue";
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
    return (
      actionProposed.value.state === ActionState.OnHold ||
      actionProposed.value.holdStatusInfluencedBy?.length > 0
    );
  else return false;
});
const actionFailed = computed(() => {
  if (actionProposed.value)
    return actionProposed.value.state === ActionState.Failed;
  else return false;
});
const actionRunning = computed(() => {
  if (actionProposed.value)
    return actionProposed.value.state === ActionState.Running;
  else return false;
});
const actionQueued = computed(() => {
  if (actionProposed.value)
    return actionProposed.value.state === ActionState.Queued;
  else return false;
});

type ActionViewWithComponent = ActionView & {
  component: FullComponent | undefined;
};

const hydrateActions = (actionList: ActionId[] | undefined) => {
  const actions = [] as ActionViewWithComponent[];
  if (actionList) {
    for (const id of actionList) {
      const _a = actionStore.actionsById.get(id);
      const a = _a as unknown as ActionViewWithComponent;
      if (a) {
        if (a.componentId) {
          a.component = componentsStore.componentsById[a.componentId];
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
      Failure: "text-destructive-600",
      Unknown: "text-warning-600",
    }[actionHistory.value.result];
  } else return undefined;
});

const resultIcon = computed(() => {
  if (actionHistory.value) {
    return {
      Success: "check-hex",
      Failure: "x-hex-outline",
      Unknown: "question-hex-outline",
    }[actionHistory.value.result] as IconNames;
  } else return "none" as IconNames;
});

const actionIconClass = (kind: ActionKind) => {
  return {
    Create: "text-success-600",
    Destroy: "text-destructive-600",
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
  return componentsStore.componentsById[props.action.componentId];
});

const emit = defineEmits<{
  (e: "add"): void;
  (e: "remove"): void;
  (e: "openMenu", mouse: MouseEvent): void;
  (e: "history", id: ActionId, tabSlug: string): void;
}>();

function onClick() {
  if (props.noInteraction) return false;
  if (component.value) {
    componentsStore.setSelectedComponentId(component.value.id);
    componentsStore.eventBus.emit("panToComponent", {
      componentId: component.value.id,
      center: true,
    });
  }
}

const isHover = computed(
  () => componentsStore.hoveredComponentId === props.action.componentId,
);

function onHoverStart() {
  if (props.noInteraction) return false;
  if (component.value) {
    componentsStore.setHoveredComponentId(component.value.id);
  }
}

function onHoverEnd() {
  if (props.noInteraction) return false;
  if (component.value) {
    componentsStore.setHoveredComponentId(null);
  }
}
</script>

<style lang="less">
@keyframes flashRed {
  from {
    background-color: transparent;
  }
  50% {
    background-color: #ef4444; //bg-destructive-400
  }
  to {
    background-color: transparent;
  }
}
.action-failed {
  animation: 0.75s ease-in flashRed;
}
</style>
