<template>
  <SiPanel
    ref="panelRef"
    remember-size-key="status-bar"
    side="bottom"
    :min-resize="0"
    :max-resize="0.8"
    :class="clsx(!panelOpen && 'h-12', themeContainerClasses)"
    :resizeable="panelOpen"
    :default-size="320"
    :min-size="280"
  >
    <TabGroup
      class="flex flex-col w-full h-full bg-neutral-900 text-white"
      as="div"
      :selected-index="selectedTab"
      @change="changeTab"
    >
      <TabList
        as="div"
        class="flex flex-row w-full justify-end h-11 flex-shrink-0"
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
        <Tab
          v-slot="{ selected }"
          :aria-hidden="!isViewMode"
          :class="[isViewMode ? '' : 'hidden']"
        >
          <StatusBarTab :selected="selected">
            <template #icon><Icon name="credit-card" /></template>
            <template #name>Costs</template>
            <template #summary>
              <StatusBarTabPill>
                <span class="font-bold">Total:&nbsp; $86,753.09</span>
              </StatusBarTabPill>
            </template>
          </StatusBarTab>
        </Tab>
        <!--
        <Tab
          v-slot="{ selected }"
          :aria-hidden="!isViewMode"
          :class="[isViewMode ? '' : 'hidden']"
        >
          <WorkflowHistoryTab :selected="selected" />
        </Tab>
        -->
        <Tab
          v-slot="{ selected }"
          :aria-hidden="!isViewMode"
          :class="[isViewMode ? '' : 'hidden']"
        >
          <FixHistoryTab :selected="selected" />
        </Tab>
        <Tab
          v-slot="{ selected }"
          :aria-hidden="!isViewMode"
          :class="[isViewMode ? '' : 'hidden']"
        >
          <ConfirmationsTab :selected="selected" />
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
          class="flex flex-col grow w-full min-h-fit text-white overflow-auto"
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
          <!-- SLA TabPanel, currently incomplete and disabled -->
          <!-- <TabPanel
            :aria-hidden="!isViewMode"
            :class="[isViewMode ? '' : 'hidden']"
            class="h-full"
          > -->
          <!-- TOOD(nick): replace with an SLA tab panel -->
          <!-- <GenericTabPanel :component-list="componentList" />
          </TabPanel> -->
          <TabPanel
            :aria-hidden="!isViewMode"
            :class="[isViewMode ? '' : 'hidden']"
            class="h-full"
          >
            <!-- TOOD(nick): replace with a Costs tab panel -->
            <GenericTabPanel :component-list="componentList" />
          </TabPanel>
          <TabPanel
            :aria-hidden="!isViewMode"
            :class="[isViewMode ? '' : 'hidden']"
            class="h-full"
          >
            <FixHistoryPanel />
          </TabPanel>
          <TabPanel
            :aria-hidden="!isViewMode"
            :class="[isViewMode ? '' : 'hidden']"
            class="h-full"
          >
            <ConfirmationsPanel />
          </TabPanel>
        </TabPanels>
      </Transition>
    </TabGroup>
  </SiPanel>
</template>

<script lang="ts" setup>
import _ from "lodash";
import { Tab, TabGroup, TabList, TabPanel, TabPanels } from "@headlessui/vue";
import { computed, ref, watch } from "vue";
import { useRoute } from "vue-router";
import clsx from "clsx";
import StatusBarTab from "@/organisms/StatusBar/StatusBarTab.vue";
import StatusBarTabPill from "@/organisms/StatusBar/StatusBarTabPill.vue";
import SiPanel from "@/atoms/SiPanel.vue";
import QualificationTabPanel from "@/organisms/StatusBarTabs/Qualification/QualificationTabPanel.vue";
import QualificationTab from "@/organisms/StatusBarTabs/Qualification/QualificationTab.vue";
import GenericTabPanel from "@/organisms/StatusBarTabs/GenericTabPanel.vue";
import Icon from "@/ui-lib/Icon.vue";
import { useComponentsStore } from "@/store/components.store";
import { useThemeContainer } from "@/ui-lib/theme_tools";
import ChangeSetTabPanel from "@/organisms/StatusBarTabs/Changes/ChangesTabPanel.vue";
import ChangeSetTab from "@/organisms/StatusBarTabs/Changes/ChangesTab.vue";
import ConfirmationsPanel from "./StatusBarTabs/Confirmations/ConfirmationsPanel.vue";
import FixHistoryTab from "./StatusBarTabs/Fix/FixHistoryTab.vue";
import FixHistoryPanel from "./StatusBarTabs/Fix/FixHistoryPanel.vue";
import ConfirmationsTab from "./StatusBarTabs/Confirmations/ConfirmationsTab.vue";

// override theme to be always dark within status bar
const { themeContainerClasses } = useThemeContainer("dark");

// Tab 0 is our phantom empty panel
const selectedTab = ref(0);
const panelOpen = ref(false);
const panelRef = ref();

const currentRoute = useRoute();
const isViewMode = computed(
  () =>
    currentRoute.name === "workspace-view" ||
    currentRoute.name === "workspace-fix",
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

const componentsStore = useComponentsStore();

const componentList = computed(() =>
  _.map(componentsStore.allComponents, (c) => ({
    id: c.id,
    name: `${c.schemaName} - ${c.displayName}`,
  })),
);

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
