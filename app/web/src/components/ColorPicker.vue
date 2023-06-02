<template>
  <span class="flex">
    <span
      :id="id ?? 'color-picker'"
      ref="pickerElement"
      :aria-required="required ?? false"
      :style="{ backgroundColor: modelValue }"
      class="block w-10 h-6 py-2 border rounded-md shadow-sm focus:outline-none sm:text-sm dark:color-white"
      :class="boxClasses"
    />
    <span class="p-1">{{ modelValue.toUpperCase() }}</span>
  </span>
</template>

<script lang="ts" setup>
import { ref, computed, onMounted } from "vue";
import Picker from "vanilla-picker";

const props = defineProps<{
  id?: string;
  required?: boolean;
  modelValue: string;
  disabled?: boolean;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", v: string): void;
  (e: "change", v: string): void;
}>();

const colorChanged = (color: { hex: string }) => {
  const colorHex = color.hex.substring(0, color.hex.length - 2);
  emit("update:modelValue", colorHex);
  emit("change", colorHex);
};

const boxClasses = computed(() => {
  const results: { [key: string]: boolean } = {};
  if (!props.disabled) {
    results["cursor-pointer"] = true;
  }
  return results;
});

const pickerElement = ref<HTMLElement | null>(null);
const picker = ref<Picker | null>(null);
onMounted(() => {
  if (!props.disabled) {
    const p = new Picker(pickerElement.value as HTMLElement);
    p.onDone = colorChanged;
    picker.value = p;
    p.setOptions({ alpha: false });
  }
});
</script>

<style lang="less">
.picker_wrapper.popup,
.picker_wrapper.popup .picker_arrow::before,
.picker_wrapper.popup .picker_arrow::after {
  background: white;
  z-index: 100;
  body.dark & {
    background: black;
  }
}

.picker_wrapper.popup {
  border-radius: 0 0.25rem 0.25rem 0.25rem;
}

.picker_editor input,
.picker_sample {
  border-radius: 0.25rem;
  overflow: hidden;
}

.picker_wrapper,
.picker_editor,
.picker_editor input,
.picker_editor input::placeholder {
  background: white;
  body.dark & {
    background: @colors-neutral-700;
    color: white;
  }
}

.picker_done button {
  background: white;
  border-radius: 0.25rem;
  body.dark & {
    background: @colors-neutral-700;
    &:hover {
      background: black;
    }
  }
}
</style>
