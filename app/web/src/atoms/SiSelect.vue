<template>
  <div
    v-tooltip.bottom="props.tooltipText ?? ''"
    :aria-label="props.tooltipText ?? ''"
  >
    <Listbox
      :id="id"
      v-slot="{ open }"
      v-model="selectedValue"
      :name="id"
      :data-test="dataTest ?? ''"
      :disabled="disabled"
      as="div"
      @keypress.space.prevent
    >
      <div class="relative">
        <ListboxButton
          :class="boxClasses"
          class="relative w-full rounded-sm shadow-sm pr-10 py-1 text-left cursor-default sm:text-sm disabled:opacity-50"
          @mouseover="changeListboxHovered(true)"
          @mouseleave="changeListboxHovered(false)"
        >
          <span class="block" :class="selectedLabelClasses">
            <template v-if="selectedLabel">
              {{ selectedLabel }}
            </template>
            <template v-else> &nbsp; </template>
          </span>

          <span
            class="absolute inset-y-0 right-0 flex items-center pr-2 pointer-events-none"
          >
            <div v-if="props.navbarMode">
              <SiArrow
                :nudge="open || listboxHovered === true"
                class="h-5 w-5 text-white"
              />
            </div>
            <div v-else>
              <SelectorIcon class="h-5 w-5 text-gray-400" aria-hidden="true" />
            </div>
          </span>
        </ListboxButton>

        <transition
          leave-active-class="transition ease-in duration-100"
          leave-from-class="opacity-100"
          leave-to-class="opacity-0"
        >
          <ListboxOptions
            :class="dropdownClasses"
            class="absolute z-10 mt-1 w-full shadow-lg max-h-60 rounded-sm py-1 text-gray-100 ring-1 ring-gray-600 ring-opacity-1 overflow-auto focus:outline-none sm:text-xs"
          >
            <ListboxOption
              v-for="(option, index) of options"
              v-slot="{ active, selected }"
              :key="index"
              :value="option.value"
              as="template"
            >
              <li
                v-if="props.navbarMode"
                :class="[
                  active ? 'text-white bg-[#2F80ED]' : 'text-gray-100',
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

              <li
                v-else
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
import { computed, ref, toRefs } from "vue";
import {
  Listbox,
  ListboxButton,
  ListboxOption,
  ListboxOptions,
} from "@headlessui/vue";
import { LabelList } from "@/api/sdf/dal/label_list";
import { CheckIcon, SelectorIcon } from "@heroicons/vue/solid";
import _ from "lodash";
import SiArrow from "@/atoms/SiArrow.vue";

export interface SelectPropsOption {
  value: string | number;
  label: string;
}

export interface SelectProps {
  options: SelectPropsOption[];
  value: string | number;
}

const listboxHovered = ref<boolean>(false);
const changeListboxHovered = (input: boolean) => {
  listboxHovered.value = input;
};

const boxClasses = computed((): Record<string, boolean> => {
  if (props.navbarMode) {
    return {
      "text-white": true,
    };
  }
  return {
    "bg-gray-900": true,
    "text-gray-100": true,
    border: true,
    "border-gray-600": true,
    "focus:outline-none": true,
    "focus:ring-indigo-200": true,
    "focus:border-indigo-200": true,
  };
});

const dropdownClasses = computed((): Record<string, boolean> => {
  if (props.navbarMode) {
    return {
      "bg-[#333333]": true,
    };
  }
  return {
    "bg-gray-900": true,
  };
});

const selectedLabelClasses = computed((): Record<string, boolean> => {
  if (props.navbarMode) {
    return {
      "text-white": true,
      "font-bold": true,
    };
  }
  return {
    "text-gray-100": true,
  };
});

const props = defineProps<{
  id: string;
  options: LabelList<string | number>;
  modelValue: string | number | null;
  disabled?: boolean;
  valueAsNumber?: boolean;
  dataTest?: string;
  tooltipText?: string;
  navbarMode?: boolean;
}>();
const emits = defineEmits(["update:modelValue", "change"]);
const { options } = toRefs(props);

const selectedLabel = computed(() => {
  let selectedOption = _.find(options.value, ["value", selectedValue.value]);
  if (_.isNull(selectedValue.value)) {
    return "";
  }
  return selectedOption?.label;
});

const selectedValue = computed<string | number>({
  get() {
    return props.modelValue ?? "";
  },
  set(value) {
    if (value === "") {
      emits("update:modelValue", null);
    } else {
      if (props.valueAsNumber) {
        emits("update:modelValue", Number(value ?? null));
      } else {
        emits("update:modelValue", value ?? null);
      }
    }
    emits("change", value ?? null);
  },
});
</script>
