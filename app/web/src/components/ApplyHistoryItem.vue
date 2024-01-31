<template>
  <Collapsible
    v-if="fixBatch"
    :defaultOpen="!props.collapse"
    extraBorderAtBottomOfContent
    contentClasses="bg-neutral-200 dark:bg-neutral-600"
  >
    <template #label>
      <div class="flex flex-row items-center truncate">
        <StatusIndicatorIcon type="fix" :status="fixBatch.status" />
        <div class="flex flex-col pl-xs">
          <div class="flex flex-row items-center gap-1">
            <div class="font-bold flex flex-row items-center">
              <div
                v-if="
                  fixBatch.status === 'success' &&
                  fixBatch.fixes.filter((f) => f.status === 'success')
                    .length === fixBatch.fixes.length
                "
                class="text-lg whitespace-nowrap"
              >
                All actions applied
              </div>
              <div v-else>
                {{
                  fixBatch.fixes.filter((f) => f.status === "success").length
                }}
                of {{ fixBatch.fixes.length }} action{{
                  fixBatch.fixes.length > 1 ? "s" : ""
                }}
                applied
              </div>
            </div>
            <span
              v-if="fixBatch.startedAt"
              :class="
                clsx(
                  'text-xs italic',
                  themeClasses('text-neutral-700', 'text-neutral-300'),
                )
              "
            >
              <Timestamp
                v-tooltip="timestampTooltip"
                class="dark:hover:text-action-300 hover:text-action-500"
                size="normal"
                relative
                showTimeIfToday
                :date="new Date(fixBatch.startedAt)"
              />
            </span>
          </div>

          <div class="truncate">
            <span class="font-bold">By: </span>
            <span class="italic">{{ fixBatch.author }}</span>
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
      </div>
    </template>
    <template #default>
      <FixCard
        v-for="(fix, fix_index) of fixBatch.fixes"
        :key="fix_index"
        :fix="fix"
        :hideTopBorder="fix_index === 0"
      />
    </template>
  </Collapsible>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import clsx from "clsx";
import {
  themeClasses,
  Timestamp,
  Collapsible,
  dateString,
} from "@si/vue-lib/design-system";
import { computed } from "vue";
import { FixBatch } from "@/store/fixes.store";
import StatusIndicatorIcon from "./StatusIndicatorIcon.vue";
import FixCard from "./FixCard.vue";

const props = defineProps<{
  fixBatch: FixBatch;
  collapse: boolean;
}>();

const timestampTooltip = computed(() => {
  if (!props.fixBatch.startedAt) return {};

  const startedStr = dateString(props.fixBatch.startedAt, "long");
  const tooltip = {
    content: `<div class="pb-xs"><span class='font-bold'>Started At:</span> ${startedStr}</div>`,
    theme: "html",
  };

  if (props.fixBatch.finishedAt) {
    const finishedStr = dateString(props.fixBatch.finishedAt, "long");
    tooltip.content += `<div><span class='font-bold'>Finished At:</span> ${finishedStr}</div>`;
  }

  return tooltip;
});

const hasCollaborators = computed(
  () => props.fixBatch.actors && props.fixBatch.actors.length > 0,
);
const collaborators = computed(() => props.fixBatch.actors || []);
const collaboratorsTooltip = computed(() => {
  const tooltip = { content: "", theme: "html" };

  collaborators.value.forEach((actor) => {
    tooltip.content += `<div>${actor}</div>`;
  });

  tooltip.content = `<div class='flex flex-col gap-2xs'>${tooltip.content}</div>`;

  return tooltip;
});
</script>
