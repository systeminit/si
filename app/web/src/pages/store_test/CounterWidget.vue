<template>
  <div class="rounded-md bg-neutral-800 p-sm">
    <Inline alignY="center">
      <div>{{ id }}</div>
      <VButton icon="plus-circle" @click="counterStore.increment" />
      <VButton icon="minus-circle" @click="counterStore.decrement" />
      <VButton icon="refresh" @click="counterStore.reset" />
      <VButton icon="trash" tone="destructive" @click="$emit('destroy')" />
      <div>Count = {{ count }}</div>
    </Inline>
  </div>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import { Inline, VButton } from "@si/vue-lib/design-system";
import { useCounterStore } from "@/store/counter.store";

const emit = defineEmits(["destroy"]);

const props = defineProps({ id: String });

const counterStore = useCounterStore();
const count = computed(() => counterStore.counter);
</script>
