<template>
  <Panel
    initialPanelType="empty"
    :panelRef="panelRef"
    :panelContainerRef="panelContainerRef"
    :initialMaximizedContainer="initialMaximizedContainer"
    :initialMaximizedFull="initialMaximizedFull"
    v-on="$listeners"
  >
    <template v-slot:content>
      <div
        class="flex flex-col w-full h-full justify-center align-middle items-center"
      >
        <img
          width="300px"
          :src="require('@/assets/images/cheech-and-chong.svg')"
        />
      </div>
    </template>
  </Panel>
</template>

<script lang="ts">
import Vue from "vue";

import Panel from "@/molecules/Panel.vue";

export default Vue.extend({
  name: "EmptyPanel",
  props: {
    panelRef: String,
    panelContainerRef: String,
    initialMaximizedFull: Boolean,
    initialMaximizedContainer: Boolean,
  },
  components: {
    Panel,
  },
});
</script>
