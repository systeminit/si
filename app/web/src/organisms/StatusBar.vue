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
            <XCircleIcon
              v-if="
                qualificationSummary === undefined ||
                qualificationSummary.failed > 0
              "
              class="text-destructive-500"
            />
            <CheckCircleIcon v-else class="text-success-500" />
          </template>
          <template #name>Qualifications</template>
          <template #summary>
            <StatusBarTabPill
              :class="
                qualificationSummary === undefined ||
                qualificationSummary.failed > 0
                  ? 'text-destructive-500 border-destructive-500'
                  : 'border-white'
              "
            >
              Total:
              <span class="font-bold ml-1">
                {{ qualificationSummary?.total ?? "-" }}</span
              >
            </StatusBarTabPill>
            <StatusBarTabPill
              class="font-bold"
              :class="
                qualificationSummary === undefined
                  ? 'bg-destructive-100 text-destructive-500 border-destructive-500'
                  : 'bg-success-100 text-success-500'
              "
            >
              <CheckCircleIcon
                v-if="qualificationSummary !== undefined"
                class="text-success-500 w-4"
              />
              <XCircleIcon v-else class="text-destructive-500 w-4" />
              <div class="pl-px">
                {{ qualificationSummary?.succeeded ?? "-" }}
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
        <TabPanel>
          <ChangeSetTabPanel />
        </TabPanel>
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
  XCircleIcon,
} from "@heroicons/vue/solid";
import SiButtonIcon from "@/atoms/SiButtonIcon.vue";
import StatusBarTab from "./StatusBar/StatusBarTab.vue";
import StatusBarTabPill from "./StatusBar/StatusBarTabPill.vue";
import { untilUnmounted } from "vuse-rx";
import { GlobalErrorService } from "@/service/global_error";
import { QualificationService } from "@/service/qualification";
import { GetSummaryResponse } from "@/service/qualification/get_summary";
import ChangeSetTab from "@/organisms/ChangeSetTab.vue";
import ChangeSetTabPanel from "@/organisms/ChangeSetTabPanel.vue";

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

const qualificationSummary = ref<GetSummaryResponse>();

// Loads data for qualifications - total, succeeded, failed
untilUnmounted(QualificationService.getSummary()).subscribe((response) => {
  if (response.error) {
    GlobalErrorService.set(response);
    // If we encounter an error, set the summary data to undefined.
    qualificationSummary.value = undefined;
    return;
  }
  // Update the qualification summary information
  qualificationSummary.value = response;
});
</script>
