<template>
  <TabGroup
    class="flex flex-col w-full bg-neutral-900 text-white border-black border-t-[1px]"
    as="div"
    :selected-index="selectedTab"
    @change="changeTab"
  >
    <TabList
      as="div"
      class="flex flex-row w-full justify-end h-11"
      :class="barClasses"
    >
      <!-- Prefix tab -->
      <Tab>
        <div aria-hidden="true" class="hidden" />
      </Tab>

      <!-- Edit tabs -->
      <Tab
        v-slot="{ selected }"
        :aria-hidden="isViewMode"
        :class="[isViewMode ? 'hidden' : '']"
      >
        <ChangeSetTab :selected="selected" />
      </Tab>
      <Tab
        v-slot="{ selected }"
        :aria-hidden="isViewMode"
        :class="[isViewMode ? 'hidden' : '']"
      >
        <QualificationTab :selected="selected" />
      </Tab>

      <!-- View tabs -->
      <Tab
        v-slot="{ selected }"
        :aria-hidden="!isViewMode"
        :class="[isViewMode ? '' : 'hidden']"
      >
        <StatusBarTab :selected="selected">
          <template #icon><BellIcon class="text-white" /></template>
          <template #name>SLA</template>
          <template #summary>
            <StatusBarTabPill class="bg-success-100 text-success-700 font-bold">
              <span>Avail:&nbsp; 100%</span>
            </StatusBarTabPill>
            <StatusBarTabPill
              class="bg-destructive-100 text-destructive-700 font-bold"
            >
              <span>Error:&nbsp; 10%</span>
            </StatusBarTabPill>
          </template>
        </StatusBarTab>
      </Tab>
      <Tab
        v-slot="{ selected }"
        :aria-hidden="!isViewMode"
        :class="[isViewMode ? '' : 'hidden']"
      >
        <StatusBarTab :selected="selected">
          <template #icon><CreditCardIcon class="text-white" /></template>
          <template #name>Costs</template>
          <template #summary>
            <StatusBarTabPill>
              <span class="font-bold">Total:&nbsp; $86,753.09</span>
            </StatusBarTabPill>
          </template>
        </StatusBarTab>
      </Tab>
      <Tab
        v-slot="{ selected }"
        :aria-hidden="!isViewMode"
        :class="[isViewMode ? '' : 'hidden']"
      >
        <StatusBarTab :selected="selected">
          <template #icon><BadgeCheckIcon class="text-white" /></template>
          <template #name>Confirmations</template>
          <template #summary>
            <StatusBarTabPill
              v-if="
                qualificationSummary?.total && qualificationSummary?.total > 0
              "
              class="border-white"
            >
              Total:
              <b class="ml-1">{{ qualificationSummary?.total }}</b>
            </StatusBarTabPill>
          </template>
        </StatusBarTab>
      </Tab>

      <!-- Tab minimization button -->
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
        <!-- Prefix panel -->
        <TabPanel aria-hidden="true" class="hidden">hidden</TabPanel>

        <!-- Edit panels -->
        <TabPanel
          :aria-hidden="isViewMode"
          :class="[isViewMode ? 'hidden' : '']"
          class="h-full"
        >
          <ChangeSetTabPanel />
        </TabPanel>
        <TabPanel
          :aria-hidden="isViewMode"
          :class="[isViewMode ? 'hidden' : '']"
          class="h-full"
        >
          <QualificationTabPanel />
        </TabPanel>

        <!-- View panels -->
        <TabPanel
          :aria-hidden="!isViewMode"
          :class="[isViewMode ? '' : 'hidden']"
          class="h-full bg-shade-100"
        />
        <TabPanel
          :aria-hidden="!isViewMode"
          :class="[isViewMode ? '' : 'hidden']"
          class="h-full bg-shade-100"
        />
        <TabPanel
          :aria-hidden="!isViewMode"
          :class="[isViewMode ? '' : 'hidden']"
          class="h-full bg-shade-100"
        />
      </TabPanels>
    </Transition>
  </TabGroup>
</template>

<script lang="ts" setup>
import { Tab, TabGroup, TabList, TabPanel, TabPanels } from "@headlessui/vue";
import { computed, ref } from "vue";
import StatusBarTab from "@/organisms/StatusBar/StatusBarTab.vue";
import StatusBarTabPill from "@/organisms/StatusBar/StatusBarTabPill.vue";
import {
  ChevronDownIcon,
  ChevronUpIcon,
  CreditCardIcon,
  BellIcon,
  BadgeCheckIcon,
} from "@heroicons/vue/solid";
import SiButtonIcon from "@/atoms/SiButtonIcon.vue";
import ChangeSetTab from "@/organisms/StatusBarTabs/ChangeSet/ChangeSetTab.vue";
import ChangeSetTabPanel from "@/organisms/StatusBarTabs/ChangeSet/ChangeSetTabPanel.vue";
import QualificationTabPanel from "@/organisms/StatusBarTabs/Qualification/QualificationTabPanel.vue";
import QualificationTab from "@/organisms/StatusBarTabs/Qualification/QualificationTab.vue";
import { useRoute } from "vue-router";
import { GetSummaryResponse } from "@/service/qualification/get_summary";
import { QualificationService } from "@/service/qualification";
import { refFrom } from "vuse-rx/src";

// Tab 0 is our phantom empty panel
const selectedTab = ref(0);
const panelOpen = ref(false);

const currentRoute = useRoute();
const isViewMode = computed(
  () =>
    currentRoute.name === "workspace-view" ||
    currentRoute.name === "workspace-runtime",
);

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

// TODO(nick): move to new home once the view tabs are moved out of here.
const qualificationSummary = refFrom<GetSummaryResponse | undefined>(
  QualificationService.getSummary(),
);
</script>
