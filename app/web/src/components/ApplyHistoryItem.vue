<template>
  <TreeNode
    v-if="actionBatch"
    :defaultOpen="!props.collapse"
    enableGroupToggle
    labelClasses="border-b border-neutral-200 dark:border-neutral-600"
    noIndentationOrLeftBorder
  >
    <template #primaryIcon>
      <StatusIndicatorIcon type="action-runner" :status="actionBatch.status" />
    </template>
    <template #label>
      <div class="flex flex-col">
        <div class="flex flex-row items-center gap-1">
          <div class="font-bold flex flex-row items-center">
            <div
              v-if="
                actionBatch.status === 'success' &&
                actionBatch.actions.filter((f) => f.status === 'success').length === actionBatch.actions.length
              "
              class="text-lg whitespace-nowrap"
            >
              All actions applied
            </div>
            <div v-else>
              {{ actionBatch.actions.filter((f) => f.status === "success").length }}
              of {{ actionBatch.actions.length }} action{{ actionBatch.actions.length > 1 ? "s" : "" }}
              applied
            </div>
          </div>
          <span
            v-if="actionBatch.startedAt"
            :class="clsx('text-xs italic', themeClasses('text-neutral-700', 'text-neutral-300'))"
          >
            <Timestamp
              v-tooltip="timestampTooltip"
              class="dark:hover:text-action-300 hover:text-action-500"
              size="normal"
              relative="standard"
              showTimeIfToday
              :date="new Date(actionBatch.startedAt)"
            />
          </span>
        </div>

        <div class="whitespace-nowrap">
          <span class="font-bold">By: </span>
          <span class="italic">{{ actionBatch.author }}</span>
          <template v-if="hasCollaborators">
            and
            <span
              v-if="collaborators.length > 1"
              v-tooltip="collaboratorsTooltip"
              class="font-bold dark:hover:text-action-300 hover:text-action-500"
            >
              {{ collaborators.length }} others
            </span>
            <span v-else class="italic">{{ collaborators[0] }}</span>
          </template>
        </div>
      </div>
    </template>
    <template #default>
      <ActionRunnerCard
        v-for="(action, action_index) of actionBatch.actions"
        :key="action_index"
        :runner="action"
        :hideTopBorder="action_index === 0"
      />
    </template>
  </TreeNode>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import clsx from "clsx";
import { themeClasses, Timestamp, dateString, TreeNode } from "@si/vue-lib/design-system";
import { computed } from "vue";
import { DeprecatedActionBatch } from "@/store/actions.store";
import StatusIndicatorIcon from "./StatusIndicatorIcon.vue";
import ActionRunnerCard from "./Actions/ActionRunnerCard.vue";

const props = defineProps<{
  actionBatch: DeprecatedActionBatch;
  collapse: boolean;
}>();

const timestampTooltip = computed(() => {
  if (!props.actionBatch.startedAt) return {};

  const startedStr = dateString(new Date(props.actionBatch.startedAt), "long");
  const tooltip = {
    content: `<div class="pb-xs"><span class='font-bold'>Started At:</span> ${startedStr}</div>`,
    theme: "html",
  };

  if (props.actionBatch.finishedAt) {
    const finishedStr = dateString(new Date(props.actionBatch.finishedAt), "long");
    tooltip.content += `<div><span class='font-bold'>Finished At:</span> ${finishedStr}</div>`;
  }

  return tooltip;
});

const hasCollaborators = computed(() => props.actionBatch.actors && props.actionBatch.actors.length > 0);
const collaborators = computed(() => props.actionBatch.actors || []);
const collaboratorsTooltip = computed(() => {
  const tooltip = { content: "", theme: "html" };

  collaborators.value.forEach((actor) => {
    tooltip.content += `<div>${actor}</div>`;
  });

  tooltip.content = `<div class='flex flex-col gap-2xs'>${tooltip.content}</div>`;

  return tooltip;
});
</script>
