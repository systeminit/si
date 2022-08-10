<template>
  <TabGroup
    class="flex flex-col w-full bg-neutral-800 text-white border-black border-t-[1px]"
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
            <ChangeSetTab />
          </template>
        </StatusBarTab>
      </Tab>
      <Tab v-slot="{ selected }">
        <StatusBarTab :selected="selected">
          <template #icon>
            <StatusIndicatorIcon
              :status="tabQualificationsIconStatus"
              :icon-type="'solid'"
            />
          </template>
          <template #name>Qualifications</template>
          <template #summary>
            <StatusBarTabPill :class="tabTotalClass">
              Total:
              <span class="font-bold ml-1">{{ tabTotalText }}</span>
            </StatusBarTabPill>
            <StatusBarTabPill class="font-bold" :class="tabSuccessClass">
              <CheckCircleIcon
                v-if="qualificationSummary !== undefined"
                class="text-success-500 w-4"
              />
              <XCircleIcon v-else class="text-destructive-500 w-4" />
              <div class="pl-px">
                {{ tabSuccessText }}
              </div>
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

<script setup lang="ts">
import { TabGroup, TabList, Tab, TabPanels, TabPanel } from "@headlessui/vue";
import { computed, ref } from "vue";
import {
  ChevronDownIcon,
  ChevronUpIcon,
  CheckCircleIcon,
  ClockIcon,
  XCircleIcon,
} from "@heroicons/vue/solid";
import SiButtonIcon from "@/atoms/SiButtonIcon.vue";
import StatusBarTab from "@/organisms/StatusBar/StatusBarTab.vue";
import StatusBarTabPill from "@/organisms/StatusBar/StatusBarTabPill.vue";
import { refFrom } from "vuse-rx";
import { QualificationService } from "@/service/qualification";
import { GetSummaryResponse } from "@/service/qualification/get_summary";
import ChangeSetTab from "@/organisms/StatusBarTabs/ChangeSet/ChangeSetTab.vue";
import ChangeSetTabPanel from "@/organisms/StatusBarTabs/ChangeSet/ChangeSetTabPanel.vue";
import QualificationTabPanel from "@/organisms/StatusBarTabs/Qualification/QualificationTabPanel.vue";
import StatusIndicatorIcon from "@/molecules/StatusIndicatorIcon.vue";

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

// Loads data for qualifications - total, succeeded, failed
const qualificationSummary = refFrom<GetSummaryResponse | undefined>(
  QualificationService.getSummary(),
);

const tabTotalClass = computed(() => {
  return qualificationSummary.value === undefined ||
    qualificationSummary.value.failed > 0
    ? "text-destructive-500 border-destructive-500"
    : "border-white";
});
const tabTotalText = computed(() => qualificationSummary.value?.total ?? "-");
const tabSuccessClass = computed(() => {
  return qualificationSummary.value === undefined
    ? "bg-destructive-100 text-destructive-500 border-destructive-500"
    : "bg-success-100 text-success-500";
});
const tabSuccessText = computed(
  () => qualificationSummary.value?.succeeded ?? "-",
);
const tabQualificationsIconStatus = computed(() =>
  qualificationSummary.value === undefined ||
  qualificationSummary.value.failed > 0
    ? "failure"
    : "success",
);
</script>
