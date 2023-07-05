<template>
  <div
    v-if="fixBatches.length === 0"
    :class="
      clsx(
        'm-6 p-3 border rounded-md h-64 flex items-center font-bold justify-around text-center',
        themeClasses('border-neutral-300', 'border-neutral-600'),
        themeClasses('text-neutral-300', 'text-neutral-600'),
      )
    "
  >
    Nothing has been applied
  </div>
  <div v-else>
    <SiSearch autoSearch class="border-b-0" />
    <Collapsible
      v-for="(fixBatch, batch_index) of fixBatches"
      :key="batch_index"
      hideBottomBorder
    >
      <template #label>
        <div class="flex flex-row flex-wrap items-center gap-1">
          <span class="font-bold flex flex-row items-center">
            <StatusIndicatorIcon type="fix" :status="fixBatch.status" />
            <div
              v-if="
                fixBatch.status === 'success' &&
                fixBatch.fixes.filter((f) => f.status === 'success').length ===
                  fixBatch.fixes.length
              "
              class="pl-xs whitespace-nowrap"
            >
              All actions applied
            </div>
            <div v-else class="pl-xs">
              {{ fixBatch.fixes.filter((f) => f.status === "success").length }}
              of {{ fixBatch.fixes.length }} action{{
                fixBatch.fixes.length > 1 ? "s" : ""
              }}
              applied
            </div>
          </span>
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
              size="mini"
              showTimeIfToday
              :date="new Date(fixBatch.startedAt)"
            />
          </span>
        </div>
      </template>
      <template #default>
        <div class="text-sm pl-8">
          <!-- Note(victor): Not 100% sure this should be removed, but it looks redundant. Confirm with mark.-->
          <!--div class="text-success-500 tracking-tight font-bold">
              {{ fixBatch.fixes.length }}
              resource{{ fixBatch.fixes.length > 1 ? "s" : "" }} fixed
            </div-->
          <div
            :class="
              clsx(
                'text-xs italic',
                themeClasses('text-neutral-700', 'text-neutral-300'),
              )
            "
          >
            <!-- <Timestamp v-if="fixBatch.finishedAt" :date="fixBatch.finishedAt" size="extended" /> -->
          </div>
          <div>By: {{ fixBatch.author }}</div>
          <div v-if="fixBatch.startedAt" class="italic">
            Started At:
            <Timestamp size="long" :date="new Date(fixBatch.startedAt)" />
          </div>
          <div v-if="fixBatch.finishedAt" class="italic">
            Finished At:
            <Timestamp size="long" :date="new Date(fixBatch.finishedAt)" />
          </div>
        </div>

        <ul class="pl-5 mt-2">
          <Collapsible
            v-for="(fix, fix_index) of fixBatch.fixes"
            :key="fix_index"
            hideBottomBorder
            textSize="sm"
            buttonClasses="py-0.5"
            :defaultOpen="false"
          >
            <template #label>
              <StatusIndicatorIcon type="fix" :status="fix.status" />
              <div class="flex flex-col">
                <div class="font-bold pl-xs">
                  {{ `${formatTitle(fix.actionKind)} ${fix.schemaName}` }}
                </div>
              </div>
            </template>
            <template #default>
              <div class="p-2 dark:text-neutral-50 text-neutral-900">
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
                <template v-else-if="fix.resource.message">
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
                </template>
                <template v-else>
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
                </template>
              </div>
            </template>
          </Collapsible>
        </ul>
      </template>
    </Collapsible>
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed } from "vue";
import clsx from "clsx";
import {
  themeClasses,
  Timestamp,
  Collapsible,
} from "@si/vue-lib/design-system";
import SiSearch from "@/components/SiSearch.vue";
import { useFixesStore } from "@/store/fixes.store";
import CodeViewer from "./CodeViewer.vue";
import StatusIndicatorIcon from "./StatusIndicatorIcon.vue";
import FixDetails from "./FixDetails.vue";

const fixesStore = useFixesStore();

const fixBatches = computed(() => _.reverse([...fixesStore.fixBatches]));

const formatTitle = (title: string) => {
  return title
    .split(" ")
    .map((t) => `${t[0]?.toUpperCase()}${t.slice(1).toLowerCase()}`)
    .join(" ");
};
</script>
