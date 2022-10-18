<template>
  <SiTabGroup :selected-index="0">
    <template #tabs>
      <SiTabHeader :key="0">FIX HISTORY</SiTabHeader>
    </template>
    <template #panels>
      <TabPanel :key="0" class="h-full overflow-auto">
        <div
          v-if="fixBatchesWithFixes.length === 0"
          :class="
            clsx(
              'm-6 p-3 border rounded-md h-64 flex items-center font-bold justify-around text-center',
              themeClasses('border-neutral-300', 'border-neutral-600'),
              themeClasses('text-neutral-300', 'text-neutral-600'),
            )
          "
        >
          No fixes have been made ... yet
        </div>
        <div v-else>
          <SiSearch auto-search class="border-b-0" />
          <SiCollapsible
            v-for="(fixBatch, batch_index) of fixBatchesWithFixes"
            :key="batch_index"
            hide-bottom-border
          >
            <template #label>
              <div class="flex flex-row items-center gap-2">
                <span class="font-bold">Fix Happened</span>
                <span
                  :class="
                    clsx(
                      'italic text-xs',
                      themeClasses('text-neutral-700', 'text-neutral-300'),
                    )
                  "
                >
                  (<Timestamp :date="fixBatch.timestamp" relative />)
                </span>
              </div>
            </template>
            <template #default>
              <div class="text-sm pl-8">
                <div class="text-success-500 tracking-tight font-bold">
                  {{ fixBatch.fixes.length }}
                  resource{{ fixBatch.fixes.length > 1 ? "s" : "" }} fixed
                </div>
                <div
                  :class="
                    clsx(
                      'text-xs italic',
                      themeClasses('text-neutral-700', 'text-neutral-300'),
                    )
                  "
                >
                  <Timestamp :date="fixBatch.timestamp" size="extended" />
                </div>
                <div>by: {{ fixBatch.author.email }}</div>
              </div>

              <ul class="pl-5 mt-2">
                <SiCollapsible
                  v-for="(fix, fix_index) of fixBatch.fixes"
                  :key="fix_index"
                  hide-bottom-border
                  text-size="sm"
                  button-classes="py-0.5"
                  :default-open="false"
                >
                  <template #label>
                    <span
                      :class="
                        clsx(
                          'text-xs',
                          themeClasses('text-neutral-700', 'text-neutral-300'),
                        )
                      "
                    >
                      {{ fix.name }}
                    </span>
                  </template>
                  <template #default>
                    <div class="p-2">
                      <CodeViewer :code="fix.output">
                        <template #title>{{ fix.name }}</template>
                      </CodeViewer>
                    </div>
                  </template>
                </SiCollapsible>
              </ul>
            </template>
          </SiCollapsible>
        </div>
      </TabPanel>
    </template>
  </SiTabGroup>
</template>

<script lang="ts" setup>
import { TabPanel } from "@headlessui/vue";
import { computed } from "vue";
import clsx from "clsx";
import { themeClasses } from "@/ui-lib/theme_tools";
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import SiSearch from "@/molecules/SiSearch.vue";
import SiCollapsible from "@/organisms/SiCollapsible.vue";
import { useFixesStore } from "@/store/fixes/fixes.store";
import Timestamp from "@/ui-lib/Timestamp.vue";
import CodeViewer from "./CodeViewer.vue";

const fixesStore = useFixesStore();

const fixBatchesWithFixes = computed(() =>
  fixesStore.allFixBatches.map((batch) => ({
    ...batch,
    fixes: fixesStore
      .fixesOnBatch(batch.id)
      .filter((fix) => fix.status === "success"),
  })),
);
</script>
