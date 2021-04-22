<template>
  <Panel
    initialPanelType="workflow"
    :panelIndex="panelIndex"
    :panelRef="panelRef"
    :panelContainerRef="panelContainerRef"
    :initialMaximizedContainer="initialMaximizedContainer"
    :initialMaximizedFull="initialMaximizedFull"
    :isVisible="isVisible"
    :isMaximizedContainerEnabled="isMaximizedContainerEnabled"
    v-on="$listeners"
  >
    <template v-slot:menuButtons>
      <div class="flex">
        <SiSelect
          size="xs"
          id="workflowPanelSelect"
          name="workflowPanelSelect"
          :options="workflowList"
          v-model="selectedWorkflow"
          class="pl-1"
        />
      </div>
      <button
        class="pl-1 focus:outline-none disabled:opacity-30"
        :disabled="!selectedWorkflow"
        @click="runThisWorkflow()"
      >
        <PlayCircleIcon size="1.1x" />
      </button>
      <div class="flex flex-row items-center pl-1">
        <div class="flex">
          <input type="checkbox" name="dryRun" v-model="dryRun" />
        </div>
        <div class="flex pl-1 text-xs">
          Dry
        </div>
      </div>
    </template>
    <template v-slot:content>
      Poop!
    </template>
  </Panel>
</template>

<script lang="ts">
import Vue from "vue";
import Panel from "@/molecules/Panel.vue";

interface Data {
  actionList: {};
}

export default Vue.extend({
  name: "AttributePanel",
  components: {
    Panel,
  },
  props: {
    panelIndex: Number,
    panelRef: String,
    panelContainerRef: String,
    initialMaximizedFull: Boolean,
    initialMaximizedContainer: Boolean,
    isVisible: Boolean,
    isMaximizedContainerEnabled: Boolean,
  },
});
</script>
