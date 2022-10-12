<template>
  <SiTabGroup :selected-index="0">
    <template #tabs>
      <SiTabHeader :key="0">FIX HISTORY</SiTabHeader>
    </template>
    <template #panels>
      <TabPanel :key="0" class="h-full overflow-auto">
        <div
          v-if="fixBatchesWithFixes.length === 0"
          class="m-6 p-3 border border-neutral-300 dark:border-neutral-600 rounded-md h-64 flex items-center text-neutral-300 dark:text-neutral-600 font-bold"
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
              <b
                >Fix Happened - {{ fixBatch.timestamp.toLocaleDateString() }}</b
              >
            </template>
            <template #default>
              <div class="text-sm pl-8">
                <div class="text-success-500 tracking-tight font-bold">
                  {{ fixBatch.fixes.length }} resources fixed
                </div>
                <div class="">by: {{ fixBatch.author.email }}</div>
              </div>

              <ul class="pl-5 mt-2">
                <SiCollapsible
                  v-for="(fix, fix_index) of fixBatch.fixes"
                  :key="fix_index"
                  hide-bottom-border
                  text-size="sm"
                  button-classes="py-0.5"
                >
                  <template #label>
                    <span class="text-xs text-gray-600 dark:text-gray-400">
                      {{ fix.name }}
                    </span>
                  </template>
                  <template #default>
                    <!-- TODO(victor): Output should have some syntax highlighting -->
                    <pre class="pl-6 text-xs whitespace-pre-line">{{
                      fix.output
                    }}</pre>
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
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import SiSearch from "@/molecules/SiSearch.vue";
import SiCollapsible from "@/organisms/SiCollapsible.vue";
import { useFixesStore } from "@/store/fixes/fixes.store";

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
