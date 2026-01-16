<template>
  <div v-tooltip.bottom="props.tooltipText ?? ''" :aria-label="props.tooltipText ?? ''">
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
          class="relative w-full rounded-sm shadow-sm pr-10 py-1 text-left cursor-default sm:text-sm disabled:opacity-50 block h-8"
          @mouseover="changeListboxHovered(true)"
          @mouseleave="changeListboxHovered(false)"
        >
          <span class="block" :class="selectedLabelClasses">
            {{ selectedLabel }}
          </span>

          <div class="absolute inset-y-0 right-0 flex items-center pr-2 pointer-events-none">
            <div v-if="props.navbarMode">
              <SiArrow :nudge="open || listboxHovered === true" class="h-5 w-5 text-white" />
            </div>
            <div v-else class="text-neutral-400">
              <Icon name="selector" />
            </div>
          </div>
        </ListboxButton>

        <transition
          leaveActiveClass="transition ease-in duration-100"
          leaveFromClass="opacity-100"
          leaveToClass="opacity-0"
        >
          <ListboxOptions
            :class="dropdownClasses"
            class="absolute z-10 mt-1 w-full shadow-lg max-h-60 rounded-sm py-1 dark:text-neutral-100 ring-1 ring-neutral-600 ring-opacity-1 overflow-auto focus:outline-none sm:text-xs"
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
                  active ? 'dark:text-white bg-action-500' : 'dark:text-neutral-100',
                  'cursor-default select-none relative py-2 pl-3 pr-9',
                ]"
              >
                <span :class="[selected ? 'font-semibold' : 'font-normal', 'block']">
                  <template v-if="option.label">
                    {{ option.label }}
                  </template>
                  <template v-else> &nbsp; </template>
                </span>

                <span
                  v-if="selected"
                  :class="[
                    active ? 'text-white' : 'text-action-300',
                    'absolute inset-y-0 right-0 flex items-center pr-4',
                  ]"
                >
                  <Icon name="check" />
                </span>
              </li>

              <li
                v-else
                :class="[
                  active ? 'text-white bg-action-600' : 'dark:text-neutral-100',
                  'cursor-default select-none relative py-2 pl-3 pr-9',
                ]"
              >
                <span :class="[selected ? 'font-semibold' : 'font-normal', 'block']">
                  <template v-if="option.label">
                    {{ option.label }}
                  </template>
                  <template v-else> &nbsp; </template>
                </span>

                <span
                  v-if="selected"
                  :class="[
                    active ? 'text-white' : 'text-action-300',
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
  </div>
</template>

<script setup lang="ts">
import { computed, ref, toRefs } from "vue";
import { Listbox, ListboxButton, ListboxOption, ListboxOptions } from "@headlessui/vue";
import * as _ from "lodash-es";
import { Icon } from "@si/vue-lib/design-system";
import { LabelList } from "@/api/sdf/dal/label_list";
import SiArrow from "./SiArrow.vue";

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
    "bg-neutral-50": true,
    "dark:bg-neutral-900": true,
    "text-neutral-900": true,
    "dark:text-neutral-50": true,
    border: true,
    "border-neutral-300": true,
    "dark:border-neutral-600": true,
    "focus:outline-none": true,
    "focus:ring-action-200": true,
    "focus:border-action-200": true,
  };
});

const dropdownClasses = computed((): Record<string, boolean> => {
  if (props.navbarMode) {
    return {
      "bg-neutral-800": true,
    };
  }
  return {
    "bg-neutral-50": true,
    "dark:bg-neutral-900": true,
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
    "dark:text-neutral-100 px-2": true,
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
  const selectedOption = _.find(options.value, ["value", selectedValue.value]);
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
