<template>
  <span ref="pickerAnchorElement" class="h-7 block">
    <Teleport to="body">
      <span
        :id="id ?? 'color-picker'"
        ref="pickerElement"
        :aria-required="required ?? false"
        :class="
          clsx(
            'absolute z-80 h-7 px-2xs flex flex-row gap-xs items-center dark:hover:text-action-300 hover:text-action-500',
            !disabled && 'cursor-pointer',
          )
        "
      >
        <span
          class="block w-10 h-6 border rounded-md shadow-sm focus:outline-none sm:text-sm dark:color-white"
          :style="{ backgroundColor: modelValue }"
        ></span>
        <span class="text-sm">{{ modelValue.toUpperCase() }}</span>
      </span>
    </Teleport>
  </span>
</template>

<script lang="ts" setup>
import { ref, onMounted, onBeforeUnmount } from "vue";
import Picker from "vanilla-picker";
import clsx from "clsx";

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

const pickerAnchorElement = ref<HTMLElement | null>(null);
const pickerElement = ref<HTMLElement | null>(null);
const picker = ref<Picker | null>(null);
const positionPickerElement = () => {
  if (pickerElement.value && pickerAnchorElement.value) {
    const rect = pickerAnchorElement.value.getBoundingClientRect();
    pickerElement.value.style.top = `${rect.top}px`;
    pickerElement.value.style.left = `${rect.left}px`;
  }
};
const positionPickerInterval = ref(); // TODO - this is definitely not the best way to do this

onMounted(() => {
  if (!props.disabled) {
    const p = new Picker(pickerElement.value as HTMLElement);
    p.onDone = colorChanged;
    picker.value = p;
    p.setOptions({ alpha: false, popup: "left" });
    positionPickerElement();
    positionPickerInterval.value = setInterval(positionPickerElement, 10);
  }
});

onBeforeUnmount(() => {
  clearInterval(positionPickerInterval.value);
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
