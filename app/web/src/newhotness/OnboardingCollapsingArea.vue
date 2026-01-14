<template>
  <div class="bg-neutral-800 rounded border border-neutral-600">
    <div
      :class="
        clsx('p-sm border-neutral-600 select-none rounded-t', isOpen && 'border-b', !blockOpening && 'cursor-pointer')
      "
      @click="tryToggleOpen"
    >
      <slot name="header" />
    </div>
    <template v-if="isOpen">
      <div class="flex flex-col p-sm gap-md">
        <slot name="body" />
      </div>
      <div class="flex flex-row justify-end gap-xs px-sm py-xs">
        <slot name="footer" />
      </div>
    </template>
  </div>
</template>

<script lang="ts" setup>
import { ref, watch } from "vue";
import clsx from "clsx";

const props = defineProps<{
  blockOpening?: boolean;
}>();

const isOpen = ref(false);

const tryToggleOpen = () => {
  if (props.blockOpening && !isOpen.value) return;
  isOpen.value = !isOpen.value;
};

watch(isOpen, () => {
  if (!isOpen.value) {
    emit("closed");
  } else {
    emit("opened");
  }
});

const emit = defineEmits<{
  (e: "closed"): void;
  (e: "opened"): void;
}>();

defineExpose({
  open: () => {
    isOpen.value = true;
  },
  close: () => {
    isOpen.value = false;
  },
});
</script>
