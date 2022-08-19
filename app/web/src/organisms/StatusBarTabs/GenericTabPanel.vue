<template>
  <div class="flex flex-row h-full w-full">
    <StatusBarTabPanelComponentList :component-list="props.componentList" />
    <div
      class="flex flex-row items-center text-center w-full h-full bg-shade-100"
    >
      <p class="w-full text-3xl text-neutral-500">
        {{ selectedComponentText }}
      </p>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import StatusBarTabPanelComponentList, {
  ComponentListItem,
} from "@/organisms/StatusBar/StatusBarTabPanelComponentList.vue";
import { SelectionService } from "@/service/selection";

const props = defineProps<{
  componentList: ComponentListItem[];
}>();

const selectedComponentId = SelectionService.useSelectedComponentId();
const selectedComponentText = computed((): string => {
  if (selectedComponentId.value) {
    return `Selected Component ID: ${selectedComponentId.value}`;
  }
  return "No Component Selected";
});
</script>
