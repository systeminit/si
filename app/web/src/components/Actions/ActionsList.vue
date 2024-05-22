<template>
  <div>
    <div
      class="flex flex-row justify-between place-items-center py-xs bg-neutral-100 dark:bg-neutral-700 text-sm"
    >
      <div class="grow-0 mx-[.66em]">
        <Icon
          name="bullet-list"
          class="attributes-panel-item__type-icon"
          size="sm"
        />
      </div>
      <div class="grow">
        {{ actionsStore.proposedActions.length }} Action(s)
      </div>
      <div class="grow-0 flex flex-row mr-xs">
        <div
          v-for="(cnt, kind) in actionsStore.countActionsByKind"
          :key="kind"
          class="flex flex-row mx-2xs p-2xs rounded dark:bg-neutral-900 bg-neutral-200"
        >
          <div class="mx-2xs">{{ cnt }}</div>
          <StatusIndicatorIcon
            type="action"
            :status="kind.toString()"
            size="sm"
          />
        </div>
      </div>
    </div>
    <div
      v-for="action in actionsStore.proposedActions"
      :key="action.id"
      :class="
        clsx(
          'border-b',
          themeClasses('border-neutral-300', 'border-neutral-700'),
        )
      "
    >
      <ActionCard
        :slim="props.slim"
        :selected="
          props.selectedActionIds && props.selectedActionIds.includes(action.id)
        "
        :action="action"
        :noInteraction="props.noInteraction"
        @click="props.clickAction && props.clickAction(action)"
        @remove="actionsStore.CANCEL([action.id])"
      />
    </div>
  </div>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { Icon, themeClasses } from "@si/vue-lib/design-system";
import { useActionsStore, ActionView } from "@/store/actions.store";
import ActionCard from "./ActionCard.vue";
import StatusIndicatorIcon from "../StatusIndicatorIcon.vue";

const actionsStore = useActionsStore();

type clickFn = (action: ActionView) => void;

const props = defineProps<{
  noInteraction?: boolean;
  selectedActionIds?: string[];
  slim?: boolean;
  clickAction?: undefined | clickFn;
}>();
</script>
