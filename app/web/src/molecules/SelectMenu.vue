<template>
  <Listbox v-model="selectedOption" :disabled="disabled" as="div">
    <div class="relative">
      <ListboxButton
        class="cursor-default relative w-full rounded-[0.1875rem] border border-neutral-300 bg-shade-0 py-[0.3125rem] pl-3 pr-10 text-left text-neutral-900 shadow-sm hover:border-neutral-400 focus:border-neutral-500 focus:outline-none focus:ring-1 focus:ring-action-500 focus:ring-offset-2 disabled:opacity-50 dark:border-neutral-600 dark:bg-neutral-800 dark:text-neutral-50"
      >
        <span class="block truncate type-regular-xs">{{
          selectedOption.label
        }}</span>
        <span
          class="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-2"
        >
          <SelectorIcon
            class="h-5 w-5 rounded-[0.1875rem] bg-neutral-300 text-shade-0"
            aria-hidden="true"
          />
        </span>
      </ListboxButton>

      <transition
        leave-active-class="transition ease-in duration-100"
        leave-from-class="opacity-100"
        leave-to-class="opacity-0"
      >
        <ListboxOptions
          class="absolute z-10 mt-1 max-h-60 w-full overflow-auto rounded-md bg-shade-0 py-1 shadow-lg ring-1 ring-black ring-opacity-5 type-regular-xs focus:outline-none dark:bg-neutral-800"
        >
          <ListboxOption
            v-for="option in options"
            :key="option.value"
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
                  selected ? 'font-semibold' : 'font-normal',
                  'block truncate',
                ]"
              >
                {{ option.label }}
              </span>

              <span
                v-if="selected"
                :class="[
                  active ? 'text-white' : 'text-action-500',
                  'absolute inset-y-0 right-0 flex items-center pr-4',
                ]"
              >
                <CheckIcon class="h-5 w-5" aria-hidden="true" />
              </span>
            </li>
          </ListboxOption>
        </ListboxOptions>
      </transition>
    </div>
  </Listbox>
</template>

<script setup lang="ts">
import { computed } from "vue";
import {
  Listbox,
  ListboxButton,
  ListboxOption,
  ListboxOptions,
} from "@headlessui/vue";
import { CheckIcon, SelectorIcon } from "@heroicons/vue/solid";

export interface Option {
  label: string;
  value: string | number;
}

const emit = defineEmits(["update:modelValue", "change"]);

const props = defineProps<{
  options: Option[];
  modelValue?: Option;
  disabled?: boolean;
}>();

const selectedOption = computed<Option>({
  get() {
    return props.modelValue ?? { label: "", value: "" };
  },
  set(value) {
    if (value.value == "") {
      emit("update:modelValue", null);
    } else {
      emit("update:modelValue", value);
    }
    emit("change", value);
  },
});
</script>
