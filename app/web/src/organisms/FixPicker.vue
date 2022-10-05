<template>
  <SiTabGroup :selected-index="0">
    <template #tabs>
      <SiTabHeader :key="0">FIX</SiTabHeader>
    </template>
    <template #dropdownitems>
      <SiDropdownItem>FIX</SiDropdownItem>
    </template>
    <template #panels>
      <TabPanel :key="0" class="h-full overflow-hidden">
        <SiSearch auto-search placeholder="search fixes" />
        <div
          class="w-full text-neutral-400 dark:text-neutral-300 text-sm p-2 border-b dark:border-neutral-600"
        >
          Select fixes from the list below to run them.
        </div>
        <div
          class="w-full text-neutral-400 dark:text-neutral-300 text-sm p-2 border-b dark:border-neutral-600 flex flex-row items-center justify-between whitespace-nowrap gap-4 overflow-auto"
        >
          <VormInput
            class="ml-2"
            type="checkbox"
            label="Select All"
            no-label
            @update:model-value="selectAll"
            >Select All</VormInput
          >
          <VButton2 icon="tools" tone="action">Fix Resources</VButton2>
        </div>
        <ul class="overflow-y-auto">
          <SiCollapsible as="li" class="w-full" content-as="ul" default-open>
            <template #label>
              <div class="flex flex-row w-full items-center justify-between">
                <div class="mr-2 whitespace-nowrap">Resources</div>
                <div
                  class="py-1 px-2 rounded whitespace-nowrap flex flex-row items-center text-destructive-500 bg-destructive-50 dark:text-destructive-100 dark:bg-destructive-500"
                >
                  <Icon
                    name="tools"
                    size="xs"
                    class="text-destructive-500 dark:text-destructive-100"
                  />
                  {{ fixes.length }}
                </div>
              </div>
            </template>
            <template #default>
              <li v-for="fix in fixes" :key="fix.id">
                <FixSprite
                  :fix="fix"
                  :selected="fixSelection[fix.id]"
                  @toggle="
                    (c) => {
                      fixSelection[fix.id] = c;
                    }
                  "
                />
              </li>
            </template>
          </SiCollapsible>
        </ul>
      </TabPanel>
    </template>
  </SiTabGroup>
</template>

<script lang="ts" setup>
import _ from "lodash";
import { TabPanel } from "@headlessui/vue";
import { reactive } from "vue";
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import SiDropdownItem from "@/atoms/SiDropdownItem.vue";
import SiSearch from "@/molecules/SiSearch.vue";
import Icon from "@/ui-lib/Icon.vue";
import VormInput from "@/ui-lib/forms/VormInput.vue";
import VButton2 from "@/ui-lib/VButton2.vue";
import FixSprite from "@/molecules/FixSprite.vue";
import SiCollapsible from "./SiCollapsible.vue";

const selectAll = (checked: boolean) => {
  _.each(fixSelection, (_v, k) => {
    fixSelection[k] = checked;
  });
};

const fixes = [
  {
    id: 1,
    name: "This is a fix!",
    recommendation:
      "this is what we recommend you do - just fix this thing and you will be all good",
  },
  {
    id: 2,
    name: "Also a fix.",
    recommendation: "honestly idk, you figure it out",
  },
];

const fixSelection: Record<string, boolean> = reactive({
  1: false,
  2: false,
});
</script>
