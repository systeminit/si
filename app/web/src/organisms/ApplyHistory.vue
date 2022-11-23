<template>
  <SiTabGroup :selected-index="0">
    <template #tabs>
      <SiTabHeader :key="0">Apply History</SiTabHeader>
    </template>
    <template #panels>
      <TabPanel :key="0" class="h-full overflow-auto">
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
          <SiSearch auto-search class="border-b-0" />
          <SiCollapsible
            v-for="(fixBatch, batch_index) of fixBatches"
            :key="batch_index"
            hide-bottom-border
          >
            <template #label>
              <div class="flex flex-row items-center gap-2">
                <span class="font-bold flex">
                  <Icon
                    v-if="['success', 'failure'].includes(fixBatch.status)"
                    :name="
                      fixBatch.status === 'success'
                        ? 'check-square'
                        : 'x-square'
                    "
                    :class="
                      fixBatch.status === 'success'
                        ? 'text-success-500'
                        : 'text-destructive-500'
                    "
                    class="pr-2"
                    size="lg"
                    :title="`Status: ${fixBatch.status}`"
                  />
                  <span
                    v-if="
                      fixBatch.fixes.filter((f) => f.status === 'success')
                        .length === fixBatch.fixes.length
                    "
                    class="mt-2"
                    >All fixes succeeded</span
                  >
                  <span v-else class="mt-2"
                    >{{
                      fixBatch.fixes.filter((f) => f.status === "success")
                        .length
                    }}
                    of {{ fixBatch.fixes.length }} fix{{
                      fixBatch.fixes.length > 1 ? "es" : ""
                    }}
                    succeeded</span
                  >
                </span>
                <span
                  :class="
                    clsx(
                      'text-xs',
                      themeClasses('text-neutral-700', 'text-neutral-300'),
                    )
                  "
                  class="mt-1"
                >
                  <Timestamp
                    size="mini"
                    :date="new Date(fixBatch.finishedAt.replace(' UTC', ''))"
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
                  <!-- <Timestamp :date="fixBatch.finishedAt" size="extended" /> -->
                </div>
                <div>by: {{ fixBatch.author }}</div>
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
                    <HealthIcon
                      :health="fix.resource.status"
                      :message="
                        [
                          `${formatTitle(fix.action)} ${fix.schemaName}`,
                          fix.resource.message ?? '',
                        ].filter((f) => f.length > 0)
                      "
                      :view-details="fix.resource.logs"
                      class="ml-3"
                    />
                  </template>
                  <template #default>
                    <div class="p-2">
                      <CodeViewer
                        v-if="fix.resource.data"
                        :code="JSON.stringify(fix.resource.data, null, 2)"
                        class="dark:text-neutral-50 text-neutral-900"
                      >
                        <template #title> </template>
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
import HealthIcon from "@/molecules/HealthIcon.vue";
import Icon from "@/ui-lib/icons/Icon.vue";
import CodeViewer from "./CodeViewer.vue";

const fixesStore = useFixesStore();

const fixBatches = computed(() => fixesStore.allFinishedFixBatches);

const formatTitle = (title: string) => {
  return title
    .split(" ")
    .map((t) => t[0].toUpperCase() + t.slice(1).toLowerCase())
    .join(" ");
};
</script>
