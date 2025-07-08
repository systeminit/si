<template>
  <div class="flex flex-row gap-xs">
    <slot name="a" :selected="isA" :toggle="toggle"> </slot>
    <slot name="b" :selected="isB" :toggle="toggle"> </slot>
  </div>
</template>

<script setup lang="ts">
import { computed, watch } from "vue";
import { useToggle } from "../logic_composables/toggle_containers";

const openState = useToggle();

const A = Symbol("A");
const B = Symbol("B");

const props = defineProps<{
  aOrB: boolean;
}>();

watch(
  () => props.aOrB,
  () => {
    if (props.aOrB && !openState.open.value) openState.open.value = true;
    if (!props.aOrB && openState.open.value) openState.open.value = false;
  },
  { immediate: true },
);

const _aOrB = computed(() => (openState.open.value ? A : B));

const isA = computed(() => _aOrB.value === A);
const isB = computed(() => _aOrB.value === B);

const toggle = (e?: Event) => {
  emit("toggle");
  openState.toggle(e);
};

const emit = defineEmits<{
  (e: "toggle"): void;
}>();

defineExpose({
  aOrB: _aOrB,
  toggle,
  isA,
  isB,
  A,
  B,
});
</script>
