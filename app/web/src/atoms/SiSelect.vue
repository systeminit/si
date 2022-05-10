<template>
  <div v-tooltip.bottom="props.tooltipText" :aria-label="props.tooltipText">
    <Listbox
      :id="id"
      v-model="selectedValue"
      :name="id"
      :data-test="dataTest"
      :disabled="disabled"
      as="div"
      @keypress.space.prevent
    >
      <div class="mt-1 relative">
        <ListboxButton
          class="bg-gray-900 text-gray-100 relative w-full border border-gray-600 rounded-sm shadow-sm pl-3 pr-10 py-2 text-left cursor-default focus:outline-none focus:ring-1 focus:ring-indigo-200 focus:border-indigo-200 sm:text-sm disabled:opacity-50"
        >
          <span class="block text-gray-100">
            <template v-if="selectedLabel">
              {{ selectedLabel }}
            </template>
            <template v-else> &nbsp; </template>
          </span>
          <span
            class="absolute inset-y-0 right-0 flex items-center pr-2 pointer-events-none"
          >
            <SelectorIcon class="h-5 w-5 text-gray-400" aria-hidden="true" />
          </span>
        </ListboxButton>

        <transition
          leave-active-class="transition ease-in duration-100"
          leave-from-class="opacity-100"
          leave-to-class="opacity-0"
        >
          <ListboxOptions
            class="absolute z-10 mt-1 w-full bg-gray-900 shadow-lg max-h-60 rounded-sm py-1 text-gray-100 ring-1 ring-gray-600 ring-opacity-1 overflow-auto focus:outline-none sm:text-xs"
          >
            <ListboxOption
              v-for="(option, index) of options"
              v-slot="{ active, selected }"
              :key="index"
              :value="option.value"
              as="template"
            >
              <li
                :class="[
                  active ? 'text-white bg-indigo-600' : 'text-gray-100',
                  'cursor-default select-none relative py-2 pl-3 pr-9',
                ]"
              >
                <span
                  :class="[selected ? 'font-semibold' : 'font-normal', 'block']"
                >
                  <template v-if="option.label">
                    {{ option.label }}
                  </template>
                  <template v-else> &nbsp; </template>
                </span>

                <span
                  v-if="selected"
                  :class="[
                    active ? 'text-white' : 'text-indigo-300',
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
  </div>
</template>

<script setup lang="ts">
import { PropType, computed, toRefs } from "vue";
import {
  Listbox,
  ListboxButton,
  ListboxOption,
  ListboxOptions,
} from "@headlessui/vue";
import { CheckIcon, SelectorIcon } from "@heroicons/vue/solid";
import _ from "lodash";

export interface SelectPropsOption {
  value: unknown;
  label: unknown;
}

export interface SelectProps {
  size: "xs" | "sm" | "base" | "lg";
  options: SelectPropsOption[];

  // TODO: fix below
  // eslint-disable-next-line @typescript-eslint/ban-types
  value: string | null | Object | number;
}

const props = defineProps({
  styling: {
    type: Object as PropType<Record<string, unknown>>,
    default: null,
  },
  id: {
    type: String,
    required: true,
  },
  size: {
    type: String as () => SelectProps["size"],
    default: "base",
  },
  options: {
    type: Array as () => SelectProps["options"],
    required: true,
  },
  modelValue: {
    type: [String, Object, Number],
    default: "",
    required: false,
  },
  disabled: {
    type: Boolean,
    default: false,
  },
  valueAsNumber: {
    type: Boolean,
    default: false,
  },
  dataTest: {
    type: String,
    default: "",
    required: false,
  },
  tooltipText: {
    type: String,
    default: "",
    required: false,
  },
});
const emits = defineEmits(["update:modelValue", "change"]);
const { options } = toRefs(props);

const selectedLabel = computed(() => {
  let selectedOption = _.find(options.value, ["value", selectedValue.value]);
  if (_.isNull(selectedValue.value)) {
    return "";
  }
  return selectedOption?.label;
});

const selectedValue = computed<string | object | number | boolean | null>({
  get() {
    return props.modelValue;
  },
  set(value) {
    if (value === "") {
      emits("update:modelValue", null);
    } else {
      if (props.valueAsNumber) {
        emits("update:modelValue", Number(value));
      } else {
        emits("update:modelValue", value);
      }
    }
    emits("change", value);
  },
});
</script>
