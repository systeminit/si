<template>
  <Collapsible
    v-if="fixBatch"
    :defaultOpen="!props.collapse"
    hideBottomBorderWhenOpen
    extraBorderAtBottomOfContent
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
      <div
        v-for="(fix, fix_index) of fixBatch.fixes"
        :key="fix_index"
        class="ml-12 my-xs flex flex-row items-center"
      >
        <StatusIndicatorIcon type="fix" :status="fix.status" />
        <div class="flex flex-col pl-xs gap-xs">
          <div class="font-bold">
            {{ `${fix.displayName}` }}
          </div>
          <div class="dark:text-neutral-50 text-neutral-900">
            <div v-if="!fix.resource"></div>
            <CodeViewer
              v-else-if="fix.resource.data"
              :code="JSON.stringify(fix.resource.data, null, 2)"
              class="dark:text-neutral-50 text-neutral-900"
            >
              <template #title>
                <div class="font-bold">
                  {{ fix.resource.message ?? "Resource Code" }}
                  <FixDetails
                    v-if="fix.resource.logs && fix.resource.logs.length > 0"
                    :health="fix.resource.status"
                    :message="
                      [
                        `${formatTitle(fix.actionKind)} ${fix.schemaName}`,
                        fix.resource.message ?? '',
                      ].filter((f) => f.length > 0)
                    "
                    :details="fix.resource.logs"
                  />
                </div>
              </template>
            </CodeViewer>
            <div v-else-if="fix.resource.message" class="text-sm">
              {{ fix.resource.message }}
              <FixDetails
                v-if="fix.resource.logs && fix.resource.logs.length > 0"
                :health="fix.resource.status"
                :message="
                  [
                    `${formatTitle(fix.actionKind)} ${fix.schemaName}`,
                    fix.resource.message ?? '',
                  ].filter((f) => f.length > 0)
                "
                :details="fix.resource.logs"
              />
            </div>
            <div v-else class="text-sm">
              {{
                fix.resource.status === "ok"
                  ? "Completed successfully"
                  : "Error"
              }}
              <FixDetails
                v-if="fix.resource.logs && fix.resource.logs.length > 0"
                :health="fix.resource.status"
                :message="
                  [
                    `${formatTitle(fix.actionKind)} ${fix.schemaName}`,
                    fix.resource.message ?? '',
                  ].filter((f) => f.length > 0)
                "
                :details="fix.resource.logs"
              />
            </div>
          </div>
        </div>
      </div>
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
import CodeViewer from "./CodeViewer.vue";
import StatusIndicatorIcon from "./StatusIndicatorIcon.vue";
import FixDetails from "./FixDetails.vue";

const props = defineProps<{
  fixBatch: FixBatch;
  collapse: boolean;
}>();

const hasCollaborators = computed(
  () => props.fixBatch.actors && props.fixBatch.actors.length > 0,
);

const formatTitle = (title: string) => {
  return title
    .split(" ")
    .map((t) => `${t[0]?.toUpperCase()}${t.slice(1).toLowerCase()}`)
    .join(" ");
};

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
