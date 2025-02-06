<template>
  <span
    ref="pickerAnchorElement"
    :class="clsx('block', variant === 'box' ? 'h-8' : 'h-7')"
  >
    <Teleport to="body">
      <span
        :id="id ?? 'color-picker'"
        ref="pickerClickHitbox"
        :aria-required="required ?? false"
        :class="
          clsx(
            'z-100 absolute block',
            variant === 'box' ? 'h-8' : 'h-7',
            !disabled && pickerInView
              ? 'cursor-pointer'
              : 'pointer-events-none',
          )
        "
        @mouseover="onHover"
        @mouseout="onEndHover"
      >
      </span>
    </Teleport>
    <div
      v-if="variant === 'box'"
      :style="{ backgroundColor: modelValue }"
      :title="modelValue"
      :class="
        clsx(
          'w-8 h-8 rounded border border-neutral-600',
          hoverOrOpen &&
            'outline outline-2 outline-action-400 dark:outline-action-300',
        )
      "
    />
    <div
      v-else
      :class="
        clsx(
          'w-full h-full flex flex-row gap-xs px-2xs items-center select-none',
          hoverOrOpen
            ? themeClasses('text-action-500', 'text-action-300')
            : themeClasses('text-shade-100', 'text-shade-0'),
        )
      "
    >
      <span
        :class="
          clsx(
            'block w-10 h-6 border rounded-md shadow-sm focus:outline-none sm:text-sm',
            hoverOrOpen
              ? themeClasses('border-action-500', 'border-action-300')
              : themeClasses('border-shade-100', 'border-shade-0'),
          )
        "
        :style="{ backgroundColor: modelValue }"
      ></span>
      <span class="text-sm">{{ modelValue.toUpperCase() }}</span>
    </div>
  </span>
</template>

<script lang="ts" setup>
import { ref, onMounted, onBeforeUnmount, computed, PropType } from "vue";
import Picker from "vanilla-picker";
import clsx from "clsx";
import { themeClasses } from "../utils/theme_tools";

type ColorPickerVariant = "standard" | "box";

const props = defineProps({
  modelValue: { type: String, required: true },
  id: { type: String },
  required: { type: Boolean },
  disabled: { type: Boolean },
  scrollingParentElement: { type: HTMLElement },
  variant: {
    type: String as PropType<ColorPickerVariant>,
    default: "standard",
  },
});

const emit = defineEmits<{
  (e: "update:modelValue", v: string): void;
  (e: "change", v: string): void;
}>();

const newColor = ref(props.modelValue);
const pickerOpen = ref(false);
const hover = ref(false);

const onHover = () => {
  hover.value = true;
};
const onEndHover = () => {
  hover.value = false;
};

const hoverOrOpen = computed(() => hover.value || pickerOpen.value);

const pickerAnchorElement = ref<HTMLElement | null>(null);
const pickerClickHitbox = ref<HTMLElement | null>(null);
const picker = ref<Picker | null>(null);
const positionPickerClickHitbox = () => {
  if (pickerClickHitbox.value && pickerAnchorElement.value) {
    const rect = pickerAnchorElement.value.getBoundingClientRect();
    pickerClickHitbox.value.style.top = `${rect.top}px`;
    pickerClickHitbox.value.style.left = `${rect.left}px`;
    pickerClickHitbox.value.style.width = `${rect.width}px`;
    pickerInView.value = checkPickerInView();
  }
};
const positionPickerInterval = ref();
const pickerInView = ref(false);

function isScrolledIntoView(container: HTMLElement, element: HTMLElement) {
  const containerRect = container.getBoundingClientRect();
  const elementRect = element.getBoundingClientRect();
  return (
    elementRect.left < containerRect.right &&
    elementRect.right > containerRect.left &&
    elementRect.top < containerRect.bottom &&
    elementRect.bottom > containerRect.top
  );
}
const checkPickerInView = () => {
  if (!pickerAnchorElement.value) return false;
  else if (!props.scrollingParentElement)
    return true; // skip the check if no scrolling parent element set
  else
    return isScrolledIntoView(
      props.scrollingParentElement,
      pickerAnchorElement.value,
    );
};

onMounted(() => {
  const p = new Picker(pickerClickHitbox.value as HTMLElement);
  picker.value = p;
  p.setOptions({
    alpha: false,
    popup: "left",
    onOpen: () => {
      const goAway = () => {
        p.hide();
        if (pickerClickHitbox.value)
          pickerClickHitbox.value.style.removeProperty("pointer-events");
      };
      if (props.disabled || !checkPickerInView() || !pickerClickHitbox.value) {
        goAway();
      } else {
        pickerOpen.value = true;
        const pickerWrapperEl = pickerClickHitbox.value.getElementsByClassName(
          "picker_wrapper",
        )[0] as HTMLElement;
        if (pickerWrapperEl) {
          const arrowEl = pickerWrapperEl.getElementsByClassName(
            "picker_arrow",
          )[0] as HTMLElement;
          pickerWrapperEl.style.removeProperty("transform");
          arrowEl.classList.remove("arrow_flipped");
          const rect = pickerWrapperEl.getBoundingClientRect();
          if (rect.bottom > window.innerHeight) {
            // Too close to the bottom, flip it!
            pickerWrapperEl.style.transform = "translateY(-100%)";
            arrowEl.classList.add("arrow_flipped");
          }
          pickerWrapperEl.style.visibility = "visible";
        } else goAway();
      }
    },
    onClose: () => {
      pickerOpen.value = false;
      p.setColor(props.modelValue, true);
    },
    onChange: (color: { hex: string }) => {
      const colorHex = color.hex.substring(0, color.hex.length - 2);
      newColor.value = colorHex;
    },
    onDone: () => {
      emit("update:modelValue", newColor.value);
      emit("change", newColor.value);
      p.setColor(newColor.value, true);
    },
    color: props.modelValue,
  });
  positionPickerClickHitbox();
  positionPickerInterval.value = setInterval(positionPickerClickHitbox, 10);
});

onBeforeUnmount(() => {
  clearInterval(positionPickerInterval.value);
});
</script>

<style lang="less">
.picker_wrapper.popup.popup_left > .picker_arrow.arrow_flipped::before {
  transform: translate(-28px, 289px) skew(-45deg);
}

.picker_wrapper.popup.layout_default.picker_wrapper {
  visibility: hidden;
}

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
