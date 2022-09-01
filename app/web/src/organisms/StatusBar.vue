<template>
  <SiPanel
    ref="panelRef"
    remember-size-key="status-bar"
    side="bottom"
    :min-resize="0"
    :max-resize="0.8"
    size-classes="h-11 bar"
    :resizeable="false"
    :fixed-default-size="320"
  >
    <TabGroup
      class="flex flex-col w-full h-full bg-neutral-900 text-white border-black border-t"
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
        <Tab
          v-slot="{ selected }"
          :aria-hidden="!isViewMode"
          :class="[isViewMode ? '' : 'hidden']"
        >
          <WorkflowHistoryTab :selected="selected" />
        </Tab>
        <Tab
          v-slot="{ selected }"
          :aria-hidden="!isViewMode"
          :class="[isViewMode ? '' : 'hidden']"
        >
          <StatusBarTab :selected="selected">
            <template #icon><Icon name="check-badge" /></template>
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
            <!-- TOOD(wendy): replace with a Workflow History tab panel -->
            <WorkflowHistoryPanel />
          </TabPanel>
          <TabPanel
            :aria-hidden="!isViewMode"
            :class="[isViewMode ? '' : 'hidden']"
            class="h-full"
          >
            <!-- TOOD(nick): replace with a Confirmations tab panel -->
            <GenericTabPanel :component-list="componentList" />
          </TabPanel>
        </TabPanels>
      </Transition>
    </TabGroup>
  </SiPanel>
</template>

<script lang="ts" setup>
import { Tab, TabGroup, TabList, TabPanel, TabPanels } from "@headlessui/vue";
import { computed, onMounted, ref } from "vue";
import { useRoute } from "vue-router";
import { untilUnmounted } from "vuse-rx/src";
import StatusBarTab from "@/organisms/StatusBar/StatusBarTab.vue";
import StatusBarTabPill from "@/organisms/StatusBar/StatusBarTabPill.vue";
import SiPanel from "@/atoms/SiPanel.vue";
import ChangeSetTab from "@/organisms/StatusBarTabs/ChangeSet/ChangeSetTab.vue";
import ChangeSetTabPanel from "@/organisms/StatusBarTabs/ChangeSet/ChangeSetTabPanel.vue";
import QualificationTabPanel from "@/organisms/StatusBarTabs/Qualification/QualificationTabPanel.vue";
import QualificationTab from "@/organisms/StatusBarTabs/Qualification/QualificationTab.vue";
import { QualificationService } from "@/service/qualification";
import { ComponentService } from "@/service/component";
import { GlobalErrorService } from "@/service/global_error";
import { ComponentListItem } from "@/organisms/StatusBar/StatusBarTabPanelComponentList.vue";
import GenericTabPanel from "@/organisms/StatusBarTabs/GenericTabPanel.vue";
import Icon from "@/ui-lib/Icon.vue";
import WorkflowHistoryTab from "./StatusBarTabs/WorkflowHistory/WorkflowHistoryTab.vue";
import WorkflowHistoryPanel from "./StatusBarTabs/WorkflowHistory/WorkflowHistoryPanel.vue";

// Tab 0 is our phantom empty panel
const selectedTab = ref(0);
const panelOpen = ref(false);
const panelRef = ref();

const currentRoute = useRoute();
const isViewMode = computed(
  () =>
    currentRoute.name === "workspace-view" ||
    currentRoute.name === "workspace-runtime",
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
  panelRef.value.setCurrentMinResize(280);
  panelRef.value.setCurrentlyResizeable(true);
  panelRef.value.setSize(320);
};

const closePanel = () => {
  panelOpen.value = false;
  selectedTab.value = 0;
  panelRef.value.setCurrentMinResize(0);
  panelRef.value.setCurrentlyResizeable(false);
  panelRef.value.resetSize(false);
};

const barClasses = computed(() => {
  const result: Record<string, boolean> = {};
  if (panelOpen.value === true) {
    result["border-b"] = true;
    result["border-shade-100"] = true;
  }
  return result;
});

// TODO(nick): move these to new home(s) once the view tabs are moved out of here.
const qualificationSummary = QualificationService.useQualificationSummary();

const componentList = ref<ComponentListItem[]>([]);
untilUnmounted(ComponentService.listComponentsIdentification()).subscribe(
  (response) => {
    if (response.error) {
      GlobalErrorService.set(response);
    } else {
      const list: ComponentListItem[] = [];
      for (const identification of response.list) {
        // FIXME(nick): use the real component name. We may need a new route since other components lists
        // use identifications with labels (currently showing "default"), track qualifications or changeset
        // components.
        list.push({
          id: identification.value.componentId,
          name: `Component ${identification.value.componentId} (${identification.value.schemaName})`,
        });
      }
      componentList.value = list;
    }
  },
);

onMounted(() => {
  panelRef.value.resetSize(false);
  panelRef.value.setCurrentlyResizeable(false);
});
</script>

<style scoped>
.bar {
  color-scheme: dark;
}
</style>
