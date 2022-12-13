<template>
  <div>
    <SiSearch auto-search @search="onSearchUpdated" />
    <template v-if="!allComponents.length">
      <div class="p-2 text-neutral-500">No components</div>
    </template>
    <template v-else-if="!filteredComponents.length">
      <div class="p-2 text-neutral-500">No components matching your search</div>
    </template>
    <template v-else>
      <ul>
        <li
          v-for="component in filteredComponents"
          :key="component.id"
          class="border-b-2 dark:border-neutral-600 cursor-pointer"
          @click="emit('select', component.id)"
        >
          <span
            :class="
              props.selectedComponentId === component.id
                ? ['bg-action-500 text-white']
                : ['hover:bg-action-400 hover:text-white']
            "
            :style="{
              'border-color': component.color || colors.neutral[400],
            }"
            class="w-full px-2 py-2 border-l-8 group flex flex-row items-baseline"
          >
            <span
              class="whitespace-nowrap text-ellipsis overflow-hidden shrink leading-tight"
              >{{ component.displayName || "si-123" }}</span
            >
            <i
              :class="
                selectedComponentId === component.id
                  ? ['bg-action-500 text-white']
                  : ['text-neutral-500 group-hover:text-white']
              "
              class="text-sm pl-1 flex-none"
            >
              {{ component.schemaName }}
            </i>
          </span>
        </li>
      </ul>
    </template>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import SiSearch from "@/molecules/SiSearch.vue";
import { colors } from "@/utils/design_token_values";
import { useComponentsStore } from "@/store/components.store";

// TODO: deal with ids as numbers vs strings...

const props = defineProps<{
  selectedComponentId?: string;
}>();

const emit = defineEmits<{
  (e: "select", componentId: string): void;
}>();

const componentsStore = useComponentsStore();

const filterString = ref("");
function onSearchUpdated(newFilterString: string) {
  filterString.value = newFilterString;
}

const allComponents = computed(() => componentsStore.allComponents);

const filteredComponents = computed(() => {
  if (!filterString.value) return allComponents.value;
  const searchLower = filterString.value.toLowerCase();
  return allComponents.value.filter((item) => {
    return (
      item.displayName.toLowerCase().includes(searchLower) ||
      item.schemaName.toLowerCase().includes(searchLower)
    );
  });
});
</script>
