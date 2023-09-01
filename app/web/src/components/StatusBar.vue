<template>
  <ResizablePanel
    ref="panelRef"
    rememberSizeKey="status-bar"
    side="bottom"
    :class="clsx(!panelOpen && 'h-12', themeContainerClasses)"
    :resizeable="panelOpen"
    :defaultSize="320"
    :minSize="250"
    :maxSizeRatio="0.8"
  >
    <!-- TODO - we should replace this with our own TabGroup component eventually -->
    <TabGroup
      class="flex flex-col w-full h-full bg-neutral-900 text-white"
      as="div"
      :selectedIndex="selectedTab"
      @change="changeTab"
    >
      <TabList
        as="div"
        class="flex flex-row w-full justify-end h-12 flex-shrink-0 hover:children:bg-neutral-800 focus-visible:children:outline-none"
        :class="barClasses"
      >
        <!-- Prefix tab -->
        <Tab>
          <div aria-hidden="true" class="hidden" />
        </Tab>

        <!-- Edit mode tabs -->
        <Tab
          v-slot="{ selected }"
          :aria-hidden="isViewMode"
          :class="[isViewMode ? 'hidden' : '']"
        >
          <DiffTab :selected="selected" />
        </Tab>
        <Tab v-slot="{ selected }">
          <QualificationTab :selected="selected" />
        </Tab>

        <!-- View mode tabs -->
        <!-- SLA Tab mockup, currently disabled -->
        <!-- <Tab
          v-slot="{ selected }"
          :aria-hidden="!isViewMode"
          :class="[isViewMode ? '' : 'hidden']"
        >
          <StatusBarTab :selected="selected">
            <template #icon><Icon name="bell" /></template>
            <template #name>SLA</template>
            <template #summary>
              <StatusBarTabPill
                class="bg-success-100 text-success-700 font-bold"
              >
                <span>Avail:&nbsp; 100%</span>
              </StatusBarTabPill>
              <StatusBarTabPill
                class="bg-destructive-100 text-destructive-700 font-bold"
              >
                <span>Error:&nbsp; 10%</span>
              </StatusBarTabPill>
            </template>
          </StatusBarTab>
        </Tab> -->
        <!-- <Tab
          v-slot="{ selected }"
          :aria-hidden="!isViewMode"
          :class="[isViewMode ? '' : 'hidden']"
        >
          <StatusBarTab :selected="selected">
            <template #icon>
              <Icon name="credit-card" />
            </template>
            <template #name>Costs</template>
            <template #summary>
              <StatusBarTabPill>
                <span class="font-bold">Total:&nbsp; $86,753.09</span>
              </StatusBarTabPill>
            </template>
          </StatusBarTab>
        </Tab> -->
        <Tab
          v-slot="{ selected }"
          :aria-hidden="!isViewMode"
          :class="[isViewMode ? '' : 'hidden']"
        >
          <FixHistoryTab :selected="selected" />
        </Tab>

        <!-- Tab minimization button -->
        <div
          class="flex w-12 border-black border-l h-full items-center justify-center cursor-pointer"
          @click="togglePanel()"
        >
          <Icon :name="panelOpen ? 'chevron--down' : 'chevron--up'" />
        </div>
      </TabList>
      <Transition
        enterActiveClass="transition duration-100 ease-out"
        enterFromClass="transform scale-95 opacity-0"
        enterToClass="transform scale-100 opacity-100"
        leaveActiveClass="transition duration-75 ease-out"
        leaveFromClass="transform scale-100 opacity-100"
        leaveToClass="transform scale-95 opacity-0"
      >
        <TabPanels
          v-if="panelOpen"
          as="div"
          class="flex flex-col grow w-full min-h-fit text-white overflow-auto"
        >
          <!-- Prefix panel -->
          <TabPanel aria-hidden="true" class="hidden">hidden</TabPanel>

          <!-- Edit mode panels -->
          <TabPanel
            :aria-hidden="isViewMode"
            :class="[isViewMode ? 'hidden' : '']"
            class="h-full"
          >
            <DiffTabPanel />
          </TabPanel>
          <TabPanel class="h-full">
            <QualificationTabPanel />
          </TabPanel>

          <!-- View mode panels -->
          <!-- SLA TabPanel, currently incomplete and disabled -->
          <!-- <TabPanel
            :aria-hidden="!isViewMode"
            :class="[isViewMode ? '' : 'hidden']"
            class="h-full"
          > -->
          <!-- TOOD(nick): replace with an SLA tab panel -->
          <!-- <GenericTabPanel :component-list="componentList" />
          </TabPanel> -->
          <!-- <TabPanel
            :aria-hidden="!isViewMode"
            :class="[isViewMode ? '' : 'hidden']"
            class="h-full"
          >
            <GenericTabPanel :component-list="componentList" />
          </TabPanel> -->
          <TabPanel
            :aria-hidden="!isViewMode"
            :class="[isViewMode ? '' : 'hidden']"
            class="h-full"
          >
            <FixHistoryPanel />
          </TabPanel>
        </TabPanels>
      </Transition>
    </TabGroup>
  </ResizablePanel>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { Tab, TabGroup, TabList, TabPanel, TabPanels } from "@headlessui/vue";
import { computed, ref, watch } from "vue";
import clsx from "clsx";
import {
  Icon,
  useThemeContainer,
  ResizablePanel,
} from "@si/vue-lib/design-system";
import { useRoute } from "vue-router";
import { nilId } from "@/utils/nilId";
import { useChangeSetsStore } from "@/store/change_sets.store";
import QualificationTabPanel from "@/components/StatusBarTabs/Qualification/QualificationTabPanel.vue";
import QualificationTab from "@/components/StatusBarTabs/Qualification/QualificationTab.vue";
import DiffTabPanel from "@/components/StatusBarTabs/Diff/DiffTabPanel.vue";
import DiffTab from "@/components/StatusBarTabs/Diff/DiffTab.vue";
import FixHistoryTab from "@/components/StatusBarTabs/Fixes/FixHistoryTab.vue";
import FixHistoryPanel from "@/components/StatusBarTabs/Fixes/FixHistoryPanel.vue";

// override theme to be always dark within status bar
const { themeContainerClasses } = useThemeContainer("dark");

// Tab 0 is our phantom empty panel
const selectedTab = ref(0);
const panelOpen = ref(false);
const panelRef = ref();

const changeSetStore = useChangeSetsStore();

const isViewMode = computed(
  () => changeSetStore.selectedChangeSetId === nilId(),
);

const changeTab = (index: number) => {
  selectedTab.value = index;
  if (!panelOpen.value) openPanel();
};

const togglePanel = () => {
  if (panelOpen.value) {
    closePanel();
  } else {
    selectedTab.value = isViewMode.value ? 4 : 2;
    openPanel();
  }
};

const openPanel = () => {
  panelOpen.value = true;
};

const closePanel = () => {
  panelOpen.value = false;
  selectedTab.value = 0;
};

const barClasses = computed(() => {
  const result: Record<string, boolean> = {};
  if (panelOpen.value === true) {
    result["border-b"] = true;
    result["border-shade-100"] = true;
  }
  return result;
});

// const componentsStore = useComponentsStore();

// const componentList = computed(() =>
//   _.map(componentsStore.allComponents, (c) => ({
//     id: c.id,
//     name: `${c.schemaName} - ${c.displayName}`,
//   })),
// );

// close status bar when route changes
// TODO: probably do something smarter if tab still exists
const route = useRoute();
watch(() => route.name, closePanel);
</script>

<style scoped>
.bar {
  color-scheme: dark;
}
</style>
