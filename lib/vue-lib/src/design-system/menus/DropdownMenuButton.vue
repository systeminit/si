<template>
  <button
    ref="buttonRef"
    :class="
      clsx(
        'flex flex-row items-center p-2xs mb-[-1px] h-7',
        'font-mono text-[13px] text-left truncate relative',
        !noBorder && 'border',
        highlightWhenModelValue && modelValueLabel && 'bg-action-900',
        variant === 'navbar' && 'flex-1 font-bold min-w-[80px] max-w-fit',
        disabled
          ? [
              'cursor-not-allowed',
              themeClasses(
                'text-neutral-500 bg-caution-lines-light',
                'text-neutral-400 bg-caution-lines-dark',
              ),
            ]
          : 'cursor-pointer',
        isFocus
          ? themeClasses(
              'border-action-500 bg-shade-0',
              'border-action-300 bg-shade-100',
            )
          : [
              variant === 'navbar'
                ? 'border-neutral-600 bg-shade-100'
                : themeClasses(
                    'border-neutral-400 bg-neutral-100',
                    'border-neutral-600 bg-neutral-900',
                  ),
            ],
      )
    "
    @blur="onBlur"
    @focus="disabled ? null : onFocus()"
    @click="open"
  >
    <div
      :class="
        clsx('flex-1 truncate py-2xs pr-xs', variant === 'navbar' && 'px-2xs')
      "
    >
      <template v-if="modelValue && alwaysShowPlaceholder">
        <span class="text-neutral-400">{{ placeholder }}</span>
        {{ modelValueLabel }}
      </template>
      <template v-else-if="modelValue">{{ modelValueLabel }}</template>
      <template v-else>{{ placeholder }}</template>
    </div>
    <Icon
      :class="
        clsx(
          isFocus
            ? themeClasses('text-action-500', 'text-action-300')
            : themeClasses('text-neutral-400', 'text-neutral-600'),
          highlightWhenModelValue && modelValueLabel && 'text-neutral-200',
        )
      "
      name="input-type-select"
      size="sm"
    />
    <DropdownMenu
      ref="dropdownMenuRef"
      :anchorTo="{ $el: buttonRef }"
      overlapAnchorOnAnchorTo
      :forceAlignRight="alignRightOnAnchor"
      :matchWidthToAnchor="variant === 'standard' && !minWidthToAnchor"
      :minWidthToAnchor="variant === 'navbar' || minWidthToAnchor"
      :overlapAnchorOffset="4"
      :search="search"
      :searchFilters="searchFilters"
      @search="onSearch"
      @onClose="onClose"
    >
      <slot />
      <slot name="beforeOptions" />
      <DropdownMenuItem
        v-for="option in arrayOptionsFromProps"
        :key="option.value"
        :label="option.label"
        :checkable="checkable"
        :checked="option.value === modelValue"
        :enableSecondaryAction="enableSecondaryAction"
        :secondaryActionIcon="secondaryActionIcon"
        @secondaryAction="secondaryAction(option)"
        @select="selectOption(option)"
      />
      <slot name="afterOptions" />
    </DropdownMenu>
  </button>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import clsx from "clsx";
import { computed, PropType, ref } from "vue";
import Icon from "../icons/Icon.vue";
import DropdownMenu from "./DropdownMenu.vue";
import { themeClasses } from "../utils/theme_tools";
import { Filter } from "../general/SiSearch.vue";
import { InputOptions, OptionsAsArray } from "../forms/VormInput.vue";
import DropdownMenuItem from "./DropdownMenuItem.vue";
import { IconNames } from "../icons/icon_set";

export type DropdownMenuButtonVariant = "standard" | "navbar";

const buttonRef = ref();
const dropdownMenuRef = ref<InstanceType<typeof DropdownMenu>>();

const props = defineProps({
  modelValue: {
    type: [String, Number, Array, Boolean, null] as PropType<
      string | number | string[] | boolean | null
    >,
  },
  placeholder: { type: String },
  disabled: { type: Boolean },
  search: { type: Boolean },
  searchFilters: { type: Array<Filter> },
  checkable: { type: Boolean },
  focused: { type: Boolean },
  options: { type: [Object, Array] as PropType<InputOptions> },
  variant: {
    type: String as PropType<DropdownMenuButtonVariant>,
    default: "standard",
  },
  minWidthToAnchor: { type: Boolean },
  noBorder: { type: Boolean },
  alignRightOnAnchor: { type: Boolean },
  enableSecondaryAction: { type: Boolean },
  secondaryActionIcon: { type: String as PropType<IconNames> },
  alwaysShowPlaceholder: { type: Boolean },
  highlightWhenModelValue: { type: Boolean },
});

const arrayOptionsFromProps = computed((): OptionsAsArray => {
  /* eslint-disable consistent-return */
  if (!props.options) return [];
  if (_.isArray(props.options)) {
    if (!_.isObject(props.options[0])) {
      return _.map(props.options, (value) => ({
        value,
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        label: value.toString(),
      }));
    }
    // handle array of simple strings
    if (_.isString(props.options[0])) {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      return _.map(props.options, (value) => ({ value, label: value })) as any;
    }
    // otherwise its an array of { value, label } and we can pass through as is
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return props.options as any;
  } else if (_.isObject(props.options)) {
    // map object of options in format of { val1: label1, ... } to array of options
    return _.map(props.options, (value, key) => {
      let label;
      // if object looks like { o1val: 'o1 label', o2Val... }
      if (_.isString(value)) label = value;
      // if object looks like { o1val: { label: 'o1 label' }, o2val... }
      else if (_.isObject(value)) label = _.get(value, "label");
      label = label || key; // fallback to using the value as the label otherwise
      return {
        value: key,
        label,
      };
    });
  }
  return [];
});

const modelValueLabel = computed(() => {
  if (!props.modelValue || !arrayOptionsFromProps.value) return undefined;

  const selectedOptions = arrayOptionsFromProps.value.filter(
    (option) => option.value === props.modelValue,
  );

  if (selectedOptions.length > 0 && selectedOptions[0]?.label) {
    return selectedOptions[0].label; // TODO - for now it only gets one option, can't select multiple
  }

  return undefined;
});

const focus = ref(false);
const isFocus = computed(() => props.focused || focus.value);

function onFocus() {
  focus.value = true;
}
function onBlur() {
  focus.value = false;
}

const onClose = () => {
  if (buttonRef.value) buttonRef.value.blur();
  clearSearch();
};

const searchString = ref("");
const onSearch = (search: string) => {
  searchString.value = search.trim().toLocaleLowerCase();
};
const clearSearch = () => {
  searchString.value = "";
};

const open = () => {
  if (dropdownMenuRef.value && !props.disabled) {
    dropdownMenuRef.value.open();
  }
};

const close = () => {
  if (dropdownMenuRef.value) {
    dropdownMenuRef.value.close();
  }
};

const hovered = computed(() => dropdownMenuRef.value?.hovered);
const searchFilteringActive = computed(
  () => dropdownMenuRef.value?.searchFilteringActive,
);
const searchActiveFilters = computed(
  () => dropdownMenuRef.value?.searchActiveFilters || [],
);

// TODO(Wendy) - for now this component only supports string values
const selectOption = (option: { value: unknown; label: string }) => {
  emit("select", option.value as string);
  emit("update:modelValue", option.value as string);
};
const secondaryAction = (option: { value: unknown; label: string }) => {
  emit("secondaryAction", option as { value: string; label: string });
};
const emit = defineEmits<{
  (e: "select", value: string): void;
  (e: "update:modelValue", value: string): void;
  (e: "secondaryAction", option: { value: string; label: string }): void;
}>();

defineExpose({
  isOpen: dropdownMenuRef.value?.isOpen,
  open,
  close,
  hovered,
  searchFilteringActive,
  searchActiveFilters,
  searchString,
});
</script>
