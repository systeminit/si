<template>
  <div class="w-full h-full flex pointer-events-none">
    <div class="w-full h-full z-0 relative">
      <Viewer
        v-if="lightmode"
        :schematic-viewer-id="schematicViewerId"
        :viewer-state="viewerState"
        :viewer-event$="viewerEventObservable.viewerEvent$"
        :schematic-data="schematicData"
        :editor-context="editorContext"
        :schematic-kind="schematicKind"
        :deployment-node-selected="deploymentNodeSelected"
        :light-mode="true"
        class="pointer-events-auto"
      />
      <Viewer
        v-else
        :schematic-viewer-id="schematicViewerId"
        :viewer-state="viewerState"
        :viewer-event$="viewerEventObservable.viewerEvent$"
        :schematic-data="schematicData"
        :editor-context="editorContext"
        :schematic-kind="schematicKind"
        :deployment-node-selected="deploymentNodeSelected"
        :light-mode="false"
        class="pointer-events-auto"
      />

      <div class="absolute inset-0 z-10">
        <div class="flex flex-col h-full">
          <!-- panels -->
          <div class="flex flex-row grow">
            <SiSidebar side="left" class="pointer-events-auto dark:text-white">
              <SiChangesetForm />

              <TabGroup>
                <TabList>
                  <Tab>Assets Palette</Tab>
                  <Tab>Local Assets</Tab>
                  <Tab>Panel</Tab>
                </TabList>
                <TabPanels>
                  <TabPanel>
                    <AssetPalette />
                  </TabPanel>
                  <TabPanel>Local Assets</TabPanel>
                  <TabPanel>Panel</TabPanel>
                </TabPanels>
              </TabGroup>
            </SiSidebar>

            <!-- transparent div that flows through to the canvas -->
            <div class="grow bg-transparent h-full pointer-events-none"></div>
            <SiSidebar side="right" class="pointer-events-auto dark:text-white">
              {{ canoe }}
            </SiSidebar>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { SchematicKind } from "@/api/sdf/dal/schematic";
import Viewer from "@/organisims/SchematicViewer/Viewer.vue";
import * as VE from "@/organisims/SchematicViewer/viewer_event";
import _ from "lodash";
import { ViewerStateMachine } from "@/organisims/SchematicViewer/state_machine";
import SiSidebar from "@/atoms/SiSidebar.vue";
import { ThemeService } from "@/service/theme";
import { refFrom } from "vuse-rx/src";
import { computed } from "vue";
import { Theme } from "@/observable/theme";
import SiChangesetForm from "@/organisims/SiChangesetForm.vue";
import { TabGroup, TabPanel, TabPanels, TabList, Tab } from "@headlessui/vue";
import AssetPalette from "@/organisims/AssetPalette.vue";

const props = defineProps<{
  mutable: boolean;
}>();

const canoe = computed(() => {
  if (props.mutable) {
    return "compose canoe";
  }
  return "view canoe";
});

const schematicViewerId = _.uniqueId();
const viewerState = new ViewerStateMachine();
const viewerEventObservable = new VE.ViewerEventObservable();
const schematicData = {
  nodes: [],
  connections: [],
};
const editorContext = null;
const schematicKind = SchematicKind.Component;
const deploymentNodeSelected = null;

const theme = refFrom<Theme>(ThemeService.currentTheme());
const lightmode = computed(() => {
  return theme.value?.value == "light";
});
</script>
