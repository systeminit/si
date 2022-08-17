<template>
  <TabGroup
    :selected-index="selectedTab"
    as="div"
    class="flex flex-col w-full bg-neutral-900 text-white border-black border-t-[1px]"
    @change="changeTab"
  >
    <TabList
      :class="barClasses"
      as="div"
      class="flex flex-row w-full justify-end h-11"
    >
      <Tab>
        <div aria-hidden="true" class="hidden" />
      </Tab>
      <Tab v-slot="{ selected }">
        <ChangeSetTab :selected="selected" />
      </Tab>
      <Tab v-slot="{ selected }">
        <QualificationTab :selected="selected" />
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
        class="flex flex-col w-full h-80 min-h-fit text-white"
      >
        <TabPanel aria-hidden="true" class="hidden">hidden</TabPanel>
        <TabPanel class="h-full">
          <ChangeSetTabPanel />
        </TabPanel>
        <TabPanel class="h-full">
          <QualificationTabPanel />
        </TabPanel>
      </TabPanels>
    </Transition>
  </TabGroup>
</template>

<script lang="ts" setup>
import { Tab, TabGroup, TabList, TabPanel, TabPanels } from "@headlessui/vue";
import { computed, ref } from "vue";
import { ChevronDownIcon, ChevronUpIcon } from "@heroicons/vue/solid";
import SiButtonIcon from "@/atoms/SiButtonIcon.vue";
import ChangeSetTab from "@/organisms/StatusBarTabs/ChangeSet/ChangeSetTab.vue";
import ChangeSetTabPanel from "@/organisms/StatusBarTabs/ChangeSet/ChangeSetTabPanel.vue";
import QualificationTabPanel from "@/organisms/StatusBarTabs/Qualification/QualificationTabPanel.vue";
import QualificationTab from "@/organisms/StatusBarTabs/Qualification/QualificationTab.vue";

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
    result["border-shade-100"] = true;
  }
  return result;
});
</script>
