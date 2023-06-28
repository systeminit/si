<template>
  <Listbox v-model="selectedOptions" :disabled="disabled" as="div">
    <div class="relative">
      <ListboxButton
        class="cursor-default relative w-full rounded-[0.1875rem] border border-neutral-300 bg-shade-0 py-1.5 pl-3 pr-10 text-left text-neutral-900 shadow-sm hover:border-neutral-400 focus:border-neutral-500 focus:outline-none focus:ring-1 focus:ring-action-500 focus:ring-offset-2 disabled:opacity-50 dark:border-neutral-600 dark:bg-neutral-900 dark:text-neutral-50"
      >
        <span class="block truncate text-sm">{{ selectedLabel }}</span>
        <span
          class="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-2"
        >
          <Icon
            name="selector"
            class="h-5 w-5 rounded-[0.1875rem] bg-neutral-300 text-shade-0"
          />
        </span>
      </ListboxButton>

      <transition
        leaveActiveClass="transition ease-in duration-100"
        leaveFromClass="opacity-100"
        leaveToClass="opacity-0"
      >
        <ListboxOptions
          class="absolute z-10 mt-1 max-h-60 w-full overflow-auto rounded-md bg-shade-0 py-1 shadow-lg ring-1 ring-black ring-opacity-5 type-regular-xs focus:outline-none dark:bg-neutral-900"
        >
          <ListboxOption
            v-for="option in options"
            :key="`${option.value}`"
            v-slot="{ active, selected }"
            :value="option"
            as="template"
          >
            <li
              :class="[
                active
                  ? 'bg-action-500 text-neutral-50'
                  : 'text-neutral-900 dark:text-neutral-50',
                'cursor-default relative select-none py-2 pl-3 pr-9',
              ]"
            >
              <span
                :class="[
                  isSelected(option, selected)
                    ? 'font-semibold'
                    : 'font-normal',
                  'block truncate',
                ]"
              >
                {{ option.label }}
              </span>

              <span
                v-if="isSelected(option, selected)"
                :class="[
                  active ? 'text-white' : 'text-action-500',
                  'absolute inset-y-0 right-0 flex items-center pr-4',
                ]"
              >
                <Icon name="check" />
              </span>
            </li>
          </ListboxOption>
        </ListboxOptions>
      </transition>
    </div>
  </Listbox>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import {
  Listbox,
  ListboxButton,
  ListboxOption,
  ListboxOptions,
} from "@headlessui/vue";
import { Icon } from "@si/vue-lib/design-system";

export interface Option {
  label: string;
  value: string | number | object;
}

const emit = defineEmits(["update:modelValue", "change"]);

const props = defineProps<{
  options: Option[];
  modelValue: Option | Option[]; // to make this a multiselect, just pass in an array of Option here
  noneSelectedLabel?: string; // this is only valid in the multiple select case
  disabled?: boolean;
}>();

const isSelected = (option: Option, selected: boolean) =>
  selected ||
  ("length" in props.modelValue && props.modelValue.includes(option));

const toggleSelection = (selection: Option) => {
  if (!("length" in props.modelValue)) {
    return [];
  }

  if (props.modelValue.includes(selection)) {
    return props.modelValue.filter((option) => option !== selection);
  } else {
    return props.modelValue.concat([selection]);
  }
};

const selectedOptions = computed<Option | Option[]>({
  get() {
    return props.modelValue;
  },
  set(value) {
    if ("value" in props.modelValue && "value" in value) {
      emit("update:modelValue", value.value === "" ? null : value);
    } else if ("length" in props.modelValue && "value" in value) {
      emit("update:modelValue", toggleSelection(value as Option));
    } else {
      // should not be hit, but just in case
      emit("update:modelValue", value);
    }
    emit("change", value);
  },
});

const selectedLabel = computed<string>(() => {
  if ("length" in selectedOptions.value) {
    switch (selectedOptions.value.length) {
      case 0:
        return props.noneSelectedLabel ?? "select an option...";
      case 1:
        return selectedOptions.value[0]?.label ?? "label missing";
      default:
        return `${selectedOptions.value[0]?.label} (+${
          selectedOptions.value.length - 1
        })`;
    }
  }

  return selectedOptions.value.label;
});
</script>
