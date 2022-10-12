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
            >Select All
          </VormInput>
          <VButton2
            :disabled="selectedFixes.length < 1"
            icon="tools"
            tone="action"
            @click="fixesStore.EXECUTE_FIXES(selectedFixes)"
            >Fix Resources
          </VButton2>
        </div>
        <SiCollapsible as="div" class="w-full" content-as="ul" default-open>
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
                <span class="pl-1">{{ fixes.length }}</span>
              </div>
            </div>
          </template>
          <template #default>
            <TransitionGroup
              enter-active-class="duration-500 ease-out"
              enter-from-class="transform opacity-0"
              enter-to-class="opacity-100"
              leave-active-class="duration-500 ease-in"
              leave-from-class="opacity-100 "
              leave-to-class="transform opacity-0"
            >
              <li v-for="fix in fixes" :key="fix.id">
                <FixSprite
                  :fix="fix"
                  :selected="fixSelection[fix.id]"
                  @toggle="
                    (c) => {
                      fixSelection[fix.id.toString()] = c;
                    }
                  "
                />
              </li>
            </TransitionGroup>
          </template>
        </SiCollapsible>
      </TabPanel>
    </template>
  </SiTabGroup>
</template>

<script lang="ts" setup>
import { TabPanel } from "@headlessui/vue";
import { reactive, ref, computed, onBeforeUnmount, onBeforeMount } from "vue";
import { addSeconds } from "date-fns";
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import SiDropdownItem from "@/atoms/SiDropdownItem.vue";
import SiSearch from "@/molecules/SiSearch.vue";
import Icon from "@/ui-lib/Icon.vue";
import VormInput from "@/ui-lib/forms/VormInput.vue";
import VButton2 from "@/ui-lib/VButton2.vue";
import FixSprite from "@/molecules/FixSprite.vue";
import { useFixesStore, Fix } from "@/store/fixes/fixes.store";
import SiCollapsible from "./SiCollapsible.vue";

const selectAll = (checked: boolean) => {
  for (const fix of fixes.value) {
    fixSelection[fix.id] = checked;
  }
};

const fixesStore = useFixesStore();
const fixes = computed(() =>
  fixesStore.allFixes.filter(
    (fix: Fix) =>
      fix.finishedAt === undefined ||
      fix.finishedAt > addSeconds(currentTime.value, -3),
  ),
);
const fixSelection: Record<string, boolean> = reactive({});
const selectedFixes = computed(() => {
  return fixes.value.filter((fix) => {
    return fixSelection[fix.id] && fix.status === "unstarted";
  });
});

const currentTime = ref(new Date());
let dateIntervalId: Timeout;

onBeforeMount(() => {
  dateIntervalId = setInterval(() => {
    currentTime.value = new Date();
  }, 500);
});

onBeforeUnmount(() => {
  clearInterval(dateIntervalId);
});
</script>
