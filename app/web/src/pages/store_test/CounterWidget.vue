<template>
  <div class="rounded-md bg-neutral-800 p-sm">
    <Inline align-y="center">
      <div>{{ id }}</div>
      <VButton2 icon="plus-circle" @click="counterStore.increment" />
      <VButton2 icon="minus-circle" @click="counterStore.decrement" />
      <VButton2 icon="refresh" @click="counterStore.reset" />
      <VButton2 icon="trash" tone="destructive" @click="$emit('destroy')" />
      <div>Count = {{ count }}</div>
    </Inline>
  </div>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import { useCounterStore } from "@/store/counter.store";
import VButton2 from "@/ui-lib/VButton2.vue";
import Inline from "@/ui-lib/layout/Inline.vue";

const emit = defineEmits(["destroy"]);

const props = defineProps({ id: String });

const counterStore = useCounterStore();
const count = computed(() => counterStore.counter);
</script>
