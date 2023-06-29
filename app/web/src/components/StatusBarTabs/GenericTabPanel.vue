<template>
  <div class="flex flex-row h-full w-full">
    <StatusBarTabPanelComponentList :componentList="props.componentList" />
    <div
      class="flex flex-row items-center text-center w-full h-full bg-shade-100"
    >
      <p class="w-full text-3xl text-neutral-500">
        {{
          selectedComponentId
            ? `Selected Component ID: ${selectedComponentId}`
            : "No Component Selected"
        }}
      </p>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import StatusBarTabPanelComponentList, {
  ComponentListItem,
} from "@/components/StatusBar/StatusBarTabPanelComponentList.vue";
import { useComponentsStore } from "@/store/components.store";

const props = defineProps<{
  componentList: ComponentListItem[];
}>();

const componentsStore = useComponentsStore();
const selectedComponentId = computed(() => componentsStore.selectedComponentId);
</script>
