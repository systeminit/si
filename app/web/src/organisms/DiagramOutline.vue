<template>
  <div>
    <template v-if="!hierarchicalOrder.length">
      <div class="p-2 text-neutral-500">No components</div>
    </template>
    <ComponentTree
      :tree-data="hierarchicalOrder"
      @select="(componentId) => emit('select', componentId)"
    />
  </div>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import ComponentTree from "@/organisms/Tree.vue";
import { useComponentsStore } from "@/store/components.store";

const props = defineProps<{
  selectedComponentId?: string;
}>();

const emit = defineEmits<{
  (e: "select", componentId: string): void;
}>();

const componentsStore = useComponentsStore();

const hierarchicalOrder = computed(
  () => componentsStore.hierarchicalComponentOrder,
);
</script>
