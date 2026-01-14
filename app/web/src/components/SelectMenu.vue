<template>
  <Listbox v-model="selectedOptions" :disabled="disabledBySelfOrParent" as="div">
    <div class="relative">
      <ListboxButton
        class="cursor-default relative w-full rounded-[0.1875rem] border border-neutral-300 bg-shade-0 py-1.5 pl-3 pr-10 text-left text-neutral-900 shadow-sm hover:border-neutral-400 focus:border-neutral-500 focus:outline-none focus:ring-1 disabled:opacity-50 dark:border-neutral-600 dark:bg-neutral-900 dark:text-neutral-50"
      >
        <span class="block truncate text-sm">{{ selectedLabel }}</span>
        <span class="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-2">
          <Icon name="selector" class="h-5 w-5 rounded-[0.1875rem] bg-neutral-300 text-shade-0" />
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
          <div v-if="canFilter" :class="clsx('filter-container', `--theme-${theme}`)">
            <input
              v-model="filterString"
              class="filter-string"
              name="filterString"
              type="text"
              placeholder="Filter options"
            />
          </div>
          <template v-if="Array.isArray(filteredOptions)">
            <ListboxOption
              v-for="option in filteredOptions"
              :key="`${option.value}`"
              v-slot="{ active, selected }"
              :value="option"
              as="template"
            >
              <li
                :class="[
                  active ? 'bg-action-500 text-neutral-50' : 'text-neutral-900 dark:text-neutral-50',
                  'cursor-default relative select-none py-2 pl-3 pr-9',
                ]"
              >
                <span :class="[isSelected(option, selected) ? 'font-semibold' : 'font-normal', 'block truncate']">
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
          </template>
          <template v-if="filteredGroupOptions && Object.keys(filteredGroupOptions).length > 0">
            <ul
              v-for="[groupLabel, groupOptions] in Object.entries(filteredGroupOptions)"
              :key="groupLabel"
              class="pl-3 py-2"
            >
              <span class="uppercase text-neutral-400"> {{ groupLabel }} </span>
              <ListboxOption
                v-for="option in groupOptions"
                :key="`${option.value}`"
                v-slot="{ active, selected }"
                :value="option"
                as="template"
              >
                <li
                  :class="[
                    active ? 'bg-action-500 text-neutral-50' : 'text-neutral-900 dark:text-neutral-50',
                    'cursor-default relative select-none py-2 pl-3 pr-9',
                  ]"
                >
                  <span :class="[isSelected(option, selected) ? 'font-semibold' : 'font-normal', 'block truncate']">
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
            </ul>
          </template>
        </ListboxOptions>
      </transition>
    </div>
  </Listbox>
</template>

<script lang="ts" setup>
import { computed, toRefs, ref } from "vue";
import { Listbox, ListboxButton, ListboxOption, ListboxOptions } from "@headlessui/vue";
import { Icon, useDisabledBySelfOrParent, useTheme } from "@si/vue-lib/design-system";
import clsx from "clsx";

export interface Option {
  label: string;
  value: string | number | object;
}

export type GroupedOptions = Record<string, Option[]>;

export interface StringOption extends Option {
  value: string;
}

const emit = defineEmits(["update:modelValue", "change"]);

const props = defineProps<{
  options: Option[] | GroupedOptions;
  modelValue: Option | Option[]; // to make this a multiselect, just pass in an array of Option here
  noneSelectedLabel?: string; // this is only valid in the multiple select case
  disabled?: boolean;
  canFilter?: boolean;
}>();

const { disabled } = toRefs(props);
const { theme } = useTheme();

const filterString = ref("");

const filteredOptions = computed(() => {
  if (!Array.isArray(props.options)) return [];
  if (!filterString.value) return props.options;

  return props.options.filter((o) => o.label.includes(filterString.value));
});

const filteredGroupOptions = computed(() => {
  if (Array.isArray(props.options)) return {};
  if (!filterString.value) return props.options;

  const filtered = {} as GroupedOptions;
  const grouped = props.options;
  Object.keys(grouped).forEach((key) => {
    const options = grouped[key]?.filter((o) => o.label.includes(filterString.value));
    if (options && options.length > 0) filtered[key] = options;
  });
  return filtered;
});

const disabledBySelfOrParent = useDisabledBySelfOrParent(disabled);

const isSelected = (option: Option, selected: boolean) =>
  selected || ("length" in props.modelValue && props.modelValue.includes(option));

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
      emit("update:modelValue", toggleSelection(value));
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
        return `${selectedOptions.value[0]?.label} (+${selectedOptions.value.length - 1})`;
    }
  }

  return selectedOptions.value.label;
});
</script>

<style lang="less" scoped>
@vertical-gap: 8px;

.filter-container {
  --text-color: @colors-black;
  --text-color-error: @colors-destructive-600;
  --text-color-muted: @colors-neutral-500;
  --border-color: @colors-neutral-300;
  --bg-color: @colors-white;

  color: var(--text-color);

  &.--theme-dark {
    --text-color: @colors-white;
    --border-color: @colors-neutral-600;
    --bg-color: @colors-black;
  }

  &.--error {
    --text-color: @colors-destructive-600;
    --border-color: @colors-destructive-500;
  }

  &.--focused {
    // --border-color: @colors-action-500;
    input {
      box-shadow: none;
      outline: 2px solid @colors-action-500;
      outline-offset: -2px;
    }
  }

  &.--disabled {
    --text-color: @colors-neutral-500;
    --text-color-muted: @colors-neutral-400;
    --bg-color: @colors-neutral-100;

    &.--theme-dark {
      --text-color: @colors-neutral-400;
      --text-color-muted: @colors-neutral-500;
      --bg-color: @colors-neutral-900;
    }

    input {
      cursor: not-allowed;
      color: currentColor;
    }
  }
}

// this class is on whatever the input is, whether its input, textarea, select, etc
input.filter-string {
  width: 100%;
  border: 1px solid var(--border-color);
  border-radius: 3px;
  transition: border-color 0.15s;
  padding: 4px 12px;
  color: var(--text-color);
  font: inherit;
  background-color: var(--bg-color);

  padding: 2px 10px;
  height: 32px;

  &:hover {
    --border-color: @colors-neutral-500;
  }

  // set font size for our inputs
  // input[type='text']&,
  // input[type='number']&,
  // input[type='password']&,
  input {
    line-height: 1rem;
    font-size: 14px;

    // if font-size is at least 16 on mobile, ios will not automatically zoom in
    @media @mq-mobile-only {
      font-size: 16px;
    }
  }

  &::placeholder {
    color: var(--text-color-muted);
    font-style: italic;
  }

  // &:focus {
  //   border-color: @border-color--focus;
  // }

  // &:focus {
  //   // we have a custom focus style instead
  //    outline: none;
  // }
}
</style>
