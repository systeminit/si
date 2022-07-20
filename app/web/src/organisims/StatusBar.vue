<template>
  <TabGroup
    class="flex flex-col w-full bg-[#333333] text-white border-black border-t-[1px]"
    as="div"
    :selected-index="selectedTab"
    @change="changeTab"
  >
    <TabList
      as="div"
      class="flex flex-row w-full justify-end h-11"
      :class="barClasses"
    >
      <Tab>
        <div class="hidden" aria-hidden="true" />
      </Tab>
      <Tab v-slot="{ selected }">
        <StatusBarTab :selected="selected">
          <template #icon><ClockIcon class="text-white" /></template>
          <template #name>Changes</template>
          <template #summary>
            <StatusBarTabPill>
              Total: <span class="font-bold">&nbsp; 8</span>
            </StatusBarTabPill>
            <StatusBarTabPill class="bg-[#DCFCE7] text-[#22C55E] font-bold">
              + 3
            </StatusBarTabPill>
            <StatusBarTabPill class="bg-[#FDE8E8] text-[#F05252] font-bold">
              - 5
            </StatusBarTabPill>
          </template>
        </StatusBarTab>
      </Tab>
      <Tab v-slot="{ selected }">
        <StatusBarTab :selected="selected">
          <template #icon><CheckCircleIcon class="text-[#22C55E]" /></template>
          <template #name>Qualifications</template>
          <template #summary>
            <StatusBarTabPill>
              Total: <span class="font-bold">&nbsp; 3</span>
            </StatusBarTabPill>
            <StatusBarTabPill class="bg-[#DCFCE7] text-[#22C55E] font-bold">
              <CheckCircleIcon class="text-[#22C55E] w-4" />
              <div class="pl-px">3</div>
            </StatusBarTabPill>
          </template>
        </StatusBarTab>
      </Tab>
      <div
        class="flex w-12 border-black border-l h-full items-center justify-center cursor-pointer"
        @click="togglePanel()"
      >
        <SiButtonIcon v-if="panelOpen">
          <ChevronDownIcon />
        </SiButtonIcon>
        <SiButtonIcon v-else>
          <ChevronUpIcon />
        </SiButtonIcon>
      </div>
    </TabList>
    <Transition
      enter-active-class="transition duration-100 ease-out"
      enter-from-class="transform scale-95 opacity-0"
      enter-to-class="transform scale-100 opacity-100"
      leave-active-class="transition duration-75 ease-out"
      leave-from-class="transform scale-100 opacity-100"
      leave-to-class="transform scale-95 opacity-0"
    >
      <TabPanels
        v-if="panelOpen"
        as="div"
        class="flex flex-col w-full h-52 lg:h-80 min-h-fit text-white"
      >
        <TabPanel class="hidden" aria-hidden="true">hidden</TabPanel>
        <TabPanel> Changes </TabPanel>
        <TabPanel> Qualifications </TabPanel>
      </TabPanels>
    </Transition>
  </TabGroup>
</template>

<script setup lang="ts">
import { TabGroup, TabList, Tab, TabPanels, TabPanel } from "@headlessui/vue";
import { computed, ref } from "vue";
import {
  ChevronDownIcon,
  ChevronUpIcon,
  CheckCircleIcon,
  ClockIcon,
} from "@heroicons/vue/solid";
import SiButtonIcon from "@/atoms/SiButtonIcon.vue";
import StatusBarTab from "./StatusBar/StatusBarTab.vue";
import StatusBarTabPill from "./StatusBar/StatusBarTabPill.vue";

const panelOpen = ref(false);
// Tab 0 is our phantom empty panel
const selectedTab = ref(0);

const changeTab = (index: number) => {
  panelOpen.value = true;
  selectedTab.value = index;
};

const togglePanel = () => {
  if (panelOpen.value) {
    panelOpen.value = false;
    selectedTab.value = 0;
  } else {
    panelOpen.value = true;
    selectedTab.value = 2;
  }
};

const barClasses = computed(() => {
  const result: Record<string, boolean> = {};
  if (panelOpen.value === true) {
    result["border-b"] = true;
    result["border-black"] = true;
  }
  return result;
});
</script>
