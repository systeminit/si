<template>
  <ScrollArea>
    <template #top>
      <MultiSelectDetailsPanel />
    </template>
    <div
      class="relative w-full h-full overflow-y-auto flex flex-col items-center"
    >
      <VButton
        v-if="
          componentsStore.selectedComponentIds.length &&
          reconciliationsStore.diffsByComponentId
        "
        icon="tools"
        tone="action"
        @click="alterSimulation()"
      >
        Alter Simulation
      </VButton>
    </div>
  </ScrollArea>
</template>

<script lang="ts" setup>
import { VButton, ScrollArea } from "@si/vue-lib/design-system";
import MultiSelectDetailsPanel from "@/components/MultiSelectDetailsPanel.vue";
import { useComponentsStore } from "@/store/components.store";
import { useReconciliationsStore } from "@/store/reconciliations.store";

const componentsStore = useComponentsStore();
const reconciliationsStore = useReconciliationsStore();

const alterSimulation = () => {
  reconciliationsStore.ALTER_SIMULATION(componentsStore.selectedComponentIds);
};
</script>
