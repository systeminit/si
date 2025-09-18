<template>
  <div class="flex flex-row gap-xs">
    <slot name="a" :selected="isA" :toggle="() => toggleTo('a')"> </slot>
    <slot name="b" :selected="isB" :toggle="() => toggleTo('b')"> </slot>
    <slot name="c" :selected="isC" :toggle="() => toggleTo('c')"> </slot>
  </div>
</template>

<script setup lang="ts">
import { computed, watch } from "vue";

type TabOption = 'a' | 'b' | 'c';

const props = defineProps<{
  selectedTab: TabOption;
}>();

const emit = defineEmits<{
  (e: "toggle", tab: TabOption): void;
}>();

watch(
  () => props.selectedTab,
  (newTab) => {
    // Emit toggle when selectedTab prop changes
    emit("toggle", newTab);
  },
  { immediate: false }
);

const isA = computed(() => props.selectedTab === 'a');
const isB = computed(() => props.selectedTab === 'b');
const isC = computed(() => props.selectedTab === 'c');

const toggleTo = (tab: TabOption) => {
  if (props.selectedTab !== tab) {
    emit("toggle", tab);
  }
};

defineExpose({
  selectedTab: computed(() => props.selectedTab),
  toggleTo,
  isA,
  isB,
  isC,
});
</script>