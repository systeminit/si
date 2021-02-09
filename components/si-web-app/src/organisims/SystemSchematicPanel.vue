<template>
  <Panel
    initialPanelType="systemSchematic"
    :panelRef="panelRef"
    :panelContainerRef="panelContainerRef"
    :initialMaximizedContainer="initialMaximizedContainer"
    :initialMaximizedFull="initialMaximizedFull"
    v-on="$listeners"
  >
    <template v-slot:content>
      <div class="relative w-full h-full">
        <div class="z-20 flex flex-col absolute bg-transparent">
          <div>Swoop</div>
          <div>Monkey nuts</div>
        </div>
        <SiGrid class="z-10 absolute" />
      </div>
    </template>
  </Panel>
</template>

<script lang="ts">
import Vue from "vue";

import Panel from "@/molecules/Panel.vue";
import SiGrid from "@/atoms/SiGrid.vue";

export default Vue.extend({
  name: "SystemSchematicPanel",
  props: {
    panelRef: String,
    panelContainerRef: String,
    initialMaximizedFull: Boolean,
    initialMaximizedContainer: Boolean,
  },
  components: {
    Panel,
    SiGrid,
  },
});
</script>
