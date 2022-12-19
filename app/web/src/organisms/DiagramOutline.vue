<template>
  <div>
    <SiSearch auto-search @search="onSearchUpdated" />
    <template v-if="!hierarchicalOrder.length">
      <div class="p-2 text-neutral-500">No components</div>
    </template>
    <template v-else-if="hierarchicalOrder.length === 0 && filterString !== ''">
      <div class="p-2 text-neutral-500">No components matching your search</div>
    </template>
    <ComponentTree
      :tree-data="hierarchicalOrder"
      @select="(componentId) => emit('select', componentId)"
      @pan="(componentId) => emit('pan', componentId)"
    />
  </div>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import SiSearch from "@/molecules/SiSearch.vue";
import ComponentTree from "@/organisms/Tree.vue";
import { useComponentsStore } from "@/store/components.store";

const props = defineProps<{
  selectedComponentId?: string;
}>();

const emit = defineEmits<{
  (e: "select", componentId: string): void;
  (e: "pan", componentId: string): void;
}>();

const componentsStore = useComponentsStore();

const filterString = ref("");

function onSearchUpdated(newFilterString: string) {
  filterString.value = newFilterString;
}

const hierarchicalOrder = computed(() => {
  return componentsStore.filteredComponentTree(
    filterString.value.toLowerCase() || "",
  );
});
</script>
