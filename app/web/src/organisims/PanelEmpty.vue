<template>
  <Panel
    :initial-panel-type="PanelType.Empty"
    :panel-index="panelIndex"
    :panel-ref="panelRef"
    :panel-container-ref="panelContainerRef"
    :initial-maximized-container="initialMaximizedContainer"
    :initial-maximized-full="initialMaximizedFull"
    :is-visible="isVisible"
    :is-maximized-container-enabled="isMaximizedContainerEnabled"
  >
    <template #content>
      <div
        class="flex flex-col items-center justify-center w-full h-full align-middle"
      >
        {{ panelContainerRef }}
        {{ panelRef }}
        <img width="300" :src="cheechSvg" />
      </div>
    </template>
  </Panel>
</template>

<script setup lang="ts">
import Panel from "@/molecules/Panel.vue";
import cheechSvg from "@/assets/images/cheech-and-chong.svg";
import { PanelType } from "@/organisims/PanelTree/panel_types";

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
