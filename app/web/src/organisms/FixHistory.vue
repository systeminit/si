<template>
  <SiTabGroup :selected-index="0">
    <template #tabs>
      <SiTabHeader :key="0">FIX HISTORY</SiTabHeader>
    </template>
    <template #panels>
      <TabPanel :key="0" class="h-full overflow-auto">
        <SiSearch auto-search class="border-b-0" />

        <div>
          <SiCollapsible
            v-for="(fix, fix_index) of fixes"
            :key="fix_index"
            hide-bottom-border
          >
            <template #label>
              <b>Fix Happened - {{ fix.timestamp.toLocaleDateString() }}</b>
            </template>
            <template #default>
              <div class="text-sm pl-8">
                <div class="text-success-500 tracking-tight font-bold">
                  {{ fix.items.length }} resources fixed
                </div>
                <div class="">by: {{ fix.author }}</div>
              </div>

              <ul class="pl-5 mt-2">
                <SiCollapsible
                  v-for="(item, item_index) of fix.items"
                  :key="item_index"
                  hide-bottom-border
                  text-size="sm"
                  button-classes="py-0.5"
                >
                  <template #label>
                    <span class="text-xs text-gray-600 dark:text-gray-400">
                      {{ item.title }}
                    </span>
                  </template>
                  <template #default>
                    <!-- TODO(victor): Output should have syntactic coloring -->
                    <pre class="pl-6 text-xs whitespace-pre-line">{{
                      item.output
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
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import SiSearch from "@/molecules/SiSearch.vue";
import SiCollapsible from "@/organisms/SiCollapsible.vue";

// TODO(victor): this is mock object that should be removed eventually
const fixes = [
  {
    author: "fulano@email.com",
    timestamp: new Date(2002, 6, 30),
    items: [
      {
        title: "FugiatNullaPariatur",
        output: JSON.stringify(
          {
            ipsum: "Dolor",
            sit: 13,
          },
          null,
          2,
        ),
      },
      {
        title: "FugiatNullaPariatur",
        output: JSON.stringify(
          {
            ipsum: "Dolor",
            sit: 13,
          },
          null,
          2,
        ),
      },
      {
        title: "FugiatNullaPariatur",
        output: JSON.stringify(
          {
            ipsum: "Dolor",
            sit: 13,
          },
          null,
          2,
        ),
      },
      {
        title: "FugiatNullaPariatur",
        output: JSON.stringify(
          {
            ipsum: "Dolor",
            sit: 13,
          },
          null,
          2,
        ),
      },
      {
        title: "FugiatNullaPariatur",
        output: JSON.stringify(
          {
            ipsum: "Dolor",
            long: "This is a very long string that should not break the interface at all",
            sit: 13,
          },
          null,
          2,
        ),
      },
    ],
  },
];
</script>
