<template>
  <ActionCardLayout
    :noInteraction="noInteraction"
    :selected="selected"
    :actionFailed="actionFailed"
    :abbr="actionKindToAbbreviation(props.action.kind)"
    :description="action.kind === ActionKind.Manual ? action.description : ''"
    :componentSchemaName="action.componentSchemaName"
    :componentName="action.componentName"
    :componentId="action.componentId"
    :actor="action.actor"
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
      />
      <Icon
        v-else-if="actionOnHold"
        :class="
          clsx(
            props.action.holdStatusInfluencedBy.length > 0
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
        :class="actionIconClass(props.action.kind)"
        :name="actionIcon(props.action.kind)"
        size="sm"
      />
    </template>
    <template #interaction>
      <ConfirmHoldModal
        v-if="!props.noInteraction"
        ref="confirmRef"
        :ok="finishHold"
      />
      <DropdownMenu
        v-if="!props.noInteraction"
        ref="contextMenuRef"
        :forceAbove="false"
        forceAlignRight
      >
        <h5 class="text-neutral-400 pl-2xs">ACTIONS:</h5>
        <DropdownMenuItem
          v-if="props.action.state === ActionState.Queued"
          :onSelect="hold"
          icon="circle-stop"
          iconClass="text-warning-400"
          label="Put on hold"
        />
        <DropdownMenuItem
          v-if="props.action.state === ActionState.OnHold"
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
        <ol v-if="props.action.myDependentActions.length > 0">
          <li
            v-for="a in props.action.myDependentActions"
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
              {{ a.componentSchemaName ?? "unknown" }}
              {{ a.componentName ?? "unknown" }}
            </span>
          </li>
        </ol>
        <p v-else class="ml-xs">None</p>
        <h5 class="text-neutral-400 pl-2xs">WAITING ON:</h5>
        <ol v-if="props.action.dependentOnActions.length > 0">
          <li
            v-for="a in props.action.dependentOnActions"
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
              {{ a.componentSchemaName ?? "unknown" }}
              {{ a.componentName ?? "unknown" }}
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
    </template>
  </ActionCardLayout>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";

import {
  Icon,
  themeClasses,
  DropdownMenu,
  DropdownMenuItem,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { ActionKind, ActionState, ActionId } from "@/api/sdf/dal/action";
import {
  useActionsStore,
  actionKindToAbbreviation,
  actionIconClass,
  actionIcon,
} from "@/store/actions.store";
import ConfirmHoldModal from "@/components/Actions/ConfirmHoldModal.vue";

import DetailsPanelMenuIcon from "@/components/DetailsPanelMenuIcon.vue";
import ActionCardLayout from "./ActionCardLayout.vue";
import { ActionProposedViewWithHydratedChildren } from "./ActionsList.vue";

const actionStore = useActionsStore();

const props = defineProps<{
  action: ActionProposedViewWithHydratedChildren;
  slim?: boolean;
  selected?: boolean;
  noInteraction?: boolean;
}>();

const confirmRef = ref<InstanceType<typeof ConfirmHoldModal> | null>(null);

const hold = () => {
  const l = props.action.myDependencies?.length;
  if (l && l > 0) confirmRef.value?.open();
  else finishHold();
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
  if (props.action && "state" in props.action)
    return (
      props.action.state === ActionState.OnHold ||
      props.action.holdStatusInfluencedBy?.length > 0
    );
  else return false;
});
const actionFailed = computed(() => {
  if (props.action) return props.action.state === ActionState.Failed;
  else return false;
});
const actionRunning = computed(() => {
  if (props.action) return props.action.state === ActionState.Running;
  else return false;
});
const actionQueued = computed(() => {
  if (props.action) return props.action.state === ActionState.Queued;
  else return false;
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
