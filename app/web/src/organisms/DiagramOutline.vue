<template>
  <SiSearch auto-search @search="onSearchUpdated" />
  <template v-if="!allComponents.length">
    <div class="p-2 text-neutral-500">No components</div>
  </template>
  <template v-else-if="!filteredComponenets.length">
    <div class="p-2 text-neutral-500">No components matching your search</div>
  </template>
  <template v-else>
    <ul>
      <li
        v-for="component in filteredComponenets"
        :key="component.id"
        class="border-b-2 dark:border-neutral-600 cursor-pointer"
        @click="emit('select', parseInt(component.id))"
      >
        <span
          :class="
            selectedComponentId === parseInt(component.id)
              ? ['bg-action-500 text-white']
              : ['hover:bg-action-400 hover:text-white']
          "
          :style="{
            'border-color': component.color || colors.neutral[400],
          }"
          class="block px-2 py-2 border-l-8 group"
        >
          {{ component.subtitle || "si-123" }}
          <i
            :class="
              selectedComponentId === parseInt(component.id)
                ? ['bg-action-500 text-white']
                : ['text-neutral-500 group-hover:text-white']
            "
            class="text-sm pl-1"
          >
            {{ component.title }}
          </i>
        </span>
      </li>
    </ul>
  </template>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import SiSearch from "@/molecules/SiSearch.vue";
import { colors } from "@/utils/design_token_values";
import DiagramService2 from "@/service/diagram2";

// TODO: deal with ids as numbers vs strings...

defineProps<{
  selectedComponentId?: number;
}>();

// simply reusing the diagram data for now... this may change
const diagramData = DiagramService2.useDiagramData();

const emit = defineEmits<{
  (e: "select", componentId: number): void;
}>();

const filterString = ref("");
function onSearchUpdated(newFilterString: string) {
  filterString.value = newFilterString;
}

const allComponents = computed(() => {
  return diagramData.value?.nodes || [];
});
const filteredComponenets = computed(() => {
  if (!filterString.value) return allComponents.value;
  const searchLower = filterString.value.toLowerCase();
  return allComponents.value.filter((item) => {
    return (
      item.title.toLowerCase().includes(searchLower) ||
      item.subtitle?.toLowerCase().includes(searchLower)
    );
  });
});
</script>
