<template>
  <Panel
    :panel-index="panelIndex"
    :panel-ref="panelRef"
    :panel-container-ref="panelContainerRef"
    :initial-maximized-container="initialMaximizedContainer"
    :initial-maximized-full="initialMaximizedFull"
    :is-visible="isVisible"
    :is-maximized-container-enabled="isMaximizedContainerEnabled"
  >
    <template #content>
      I like my butt
    </template>
  </Panel>
</template>

<script setup lang="ts">
import Panel from "@/molecules/Panel.vue";
// import ViewerAttribute from "@/organisims/PanelAttribute/ViewerAttribute.vue";

// import { GlobalErrorService } from "@/service/global_error";
// import { ApiResponse } from "@/api/sdf";

// TODO: Nick, here is your panel. The switcher is fucked, but otherwise, should be good to port.

defineProps({
  panelIndex: { type: Number, required: true },
  panelRef: { type: String, required: true },
  panelContainerRef: { type: String, required: true },
  initialMaximizedFull: Boolean,
  initialMaximizedContainer: Boolean,
  isVisible: Boolean,
  isMaximizedContainerEnabled: Boolean,
});

</script>

<style scoped>
.unlocked {
  color: #c6c6c6;
}

.locked {
  color: #e3ddba;
}
</style>
