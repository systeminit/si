<!-- eslint-disable vue/no-mutating-props -->
/* This component is a bit of a monster, but it allows for really flexible forms
to be built quickly and IMO it's much nicer to import a single component and
change props than to have to import many components when building out forms.
Soon I will try to break it apart a bit using some of vue 3's new composition
tools, but realistically it's not super important as this component will not
need to change very often Note that this is tightly coupled to the
VormInputOption component and some pretty funky stuff is going on under the hood
although this is only used for dropdown, multi-checkbox, and radio inputs, and
you can pass in options as props too */

<template>
  <div
    :key="formInputId"
    ref="wrapperRef"
    :class="
      clsx(
        !noStyles && [
          computedClasses,
          compact ? 'vorm-input-compact' : 'vorm-input-standard',
          inlineLabel ? 'flex flex-row gap-xs items-center' : 'block',
        ],
      )
    "
    class="vorm-input"
  >
    <label
      v-if="!noLabel && !noStyles"
      v-tooltip="
        compact
          ? {
              content: labelTooltip,
              theme: 'instant-show',
              placement: 'left',
            }
          : null
      "
      :class="clsx('vorm-input__label', !inlineLabel && 'pb-xs block')"
      :for="formInputId"
    >
      <Icon
        v-if="disabledBySelfOrParent && !iconRight"
        :class="clsx('vorm-input__locked-icon')"
        name="lock"
      />
      <slot name="label">
        {{ label || "&nbsp;" }}{{ required || requiredWarning ? "*" : "" }}
      </slot>
    </label>
    <slot v-if="!noStyles" name="prompt">
      <div class="vorm-input__prompt pb-xs text-sm">{{ prompt }}</div>
    </slot>
    <Icon
      v-if="iconRight"
      :class="clsx('vorm-input__right-icon')"
      :name="iconRight"
      :rotate="iconRightRotate"
    />
    <div
      :class="
        clsx(
          noStyles
            ? 'h-full'
            : [
                'vorm-input__input-and-instructions-wrap',
                compact && [
                  noLabel
                    ? 'flex-1'
                    : ['flex-none', compactWide ? 'w-[70%]' : 'w-[45%]'],
                  !rename && 'min-h-[30px]',
                  'border',
                  isFocus
                    ? [
                        'z-[1]',
                        themeClasses(
                          'bg-shade-0 border-action-500',
                          'bg-shade-100 border-action-300',
                        ),
                      ]
                    : themeClasses(
                        'bg-neutral-100 border-neutral-400',
                        'bg-neutral-900 border-neutral-600',
                      ),
                  isError && 'border-destructive-600',
                ],
              ],
        )
      "
    >
      <div
        v-tooltip="
          compact
            ? {
                content: instructions,
                theme: 'instant-show',
                placement: 'left',
              }
            : null
        "
        :class="
          clsx(
            noStyles
              ? 'h-full'
              : [
                  'vorm-input__input-wrap',
                  showCautionLines
                    ? themeClasses(
                        'bg-caution-lines-light',
                        'bg-caution-lines-dark',
                      )
                    : '',
                  compact && isError && 'border-b border-destructive-600',
                ],
          )
        "
      >
        <template v-if="type === 'container'">
          <slot />
        </template>

        <template v-else-if="type === 'dropdown'">
          <select
            :id="formInputId"
            ref="inputRef"
            :class="[
              modelValue === null && !placeholderSelectable
                ? '--placeholder-selected'
                : '',
              compact
                ? 'vorm-input-compact__input vorm-input__hidden-input'
                : 'vorm-input__input cursor-pointer',
            ]"
            :disabled="disabledBySelfOrParent"
            :value="valueForSelectField"
            @blur="onBlur"
            @change="onSelectChange"
            @focus="onFocus"
          >
            <VormInputOption
              v-if="placeholder"
              :disabled="!placeholderSelectable"
              :hidden="!placeholderSelectable"
              :value="null"
            >
              {{ placeholder }}
            </VormInputOption>

            <slot />

            <VormInputOption
              v-for="(o, i) in arrayOptionsFromProps"
              :key="generateOptionKey(o, i)"
              :value="o.value"
              >{{ o.label }}
            </VormInputOption>
          </select>
          <template v-if="compact">
            <div class="vorm-input__input-value">
              {{
                arrayOptionsFromProps.find(
                  (o) => o.value === valueForSelectField,
                )?.label ?? valueForSelectField
              }}
            </div>
            <Icon
              class="absolute right-1 top-1 text-neutral-400 dark:text-neutral-600"
              name="input-type-select"
              size="sm"
            />
          </template>
        </template>

        <template v-else-if="type === 'dropdown-optgroup'">
          <select
            :id="formInputId"
            ref="inputRef"
            :class="[
              compact
                ? 'vorm-input-compact__input vorm-input__hidden-input'
                : 'vorm-input__input cursor-pointer',
            ]"
            :disabled="disabledBySelfOrParent"
            :value="valueForSelectField"
            @blur="onBlur"
            @change="onSelectChange"
            @focus="onFocus"
          >
            <optgroup
              v-for="(inner, innerLabel) in objectOptionsFromProps"
              :key="innerLabel"
              :label="innerLabel"
            >
              <VormInputOption
                v-for="(o, i) in inner"
                :key="generateOptionKey(o, i)"
                :value="o.value"
                >{{ o.label }}
              </VormInputOption>
            </optgroup>
          </select>
          <template v-if="compact">
            <div class="vorm-input__input-value">
              {{
                arrayOptionsFromProps.find(
                  (o) => o.value === valueForSelectField,
                )?.label ?? valueForSelectField
              }}
            </div>
            <Icon
              class="absolute right-1 top-1 text-neutral-400 dark:text-neutral-600"
              name="input-type-select"
              size="sm"
            />
          </template>
        </template>

        <template v-else-if="type === 'radio' || type === 'multi-checkbox'">
          <VormInputOption
            v-for="(o, i) in arrayOptionsFromProps"
            :key="generateOptionKey(o, i)"
            :disabled="disabledBySelfOrParent"
            :value="o.value"
            >{{ o.label }}
          </VormInputOption>
          <slot />
        </template>

        <template v-else-if="type === 'checkbox'">
          <input
            :id="formInputId"
            ref="inputRef"
            :checked="modelValue === checkedValue"
            :class="compact ? 'vorm-input-compact__input' : 'vorm-input__input'"
            :disabled="disabledBySelfOrParent"
            type="checkbox"
            @input="onCheckboxChange"
          />
          <label :for="formInputId" class="vorm-input__checkbox-text">
            <slot />
          </label>
        </template>

        <template v-else-if="type === 'textarea'">
          <textarea
            :id="formInputId"
            ref="inputRef"
            :class="compact ? 'vorm-input-compact__input' : 'vorm-input__input'"
            :disabled="disabledBySelfOrParent"
            :maxlength="maxLength"
            :placeholder="computedPlaceholder"
            :value="modelValueForTextArea"
            @blur="onBlur"
            @focus="onFocus"
            @input="onChange"
          />
        </template>

        <template v-else>
          <Icon
            v-if="type === 'password' && allowShowPassword && !noStyles"
            :name="isPasswordMasked ? 'show' : 'hide'"
            allow-pointer-events
            class="vorm-input__pass-show-hide-toggle"
            @click="isPasswordMasked = !isPasswordMasked"
          />
          <input
            :id="formInputId"
            ref="inputRef"
            :autocomplete="autocomplete"
            :class="
              clsx(
                noStyles
                  ? [
                      'bg-transparent border-none outline-none p-0 m-0 w-full h-full focus:ring-0',
                    ]
                  : [
                      compact
                        ? 'vorm-input-compact__input'
                        : 'vorm-input__input',
                      compact && rename && 'vorm-input-compact__input__rename',
                    ],
                noStyles && disabled && 'cursor-not-allowed',
              )
            "
            :disabled="disabledBySelfOrParent"
            :maxlength="maxLength"
            :minlength="minLength"
            :name="name"
            :passwordrules="
              type === 'password'
                ? `minlength: ${minLength}; maxlength: ${maxLength}; required: lower; required: upper; required: digit; required: special;`
                : undefined
            "
            :placeholder="computedPlaceholder"
            :step.prop="nativeInputNumberStepProp"
            :style="
              compact && rename
                ? `font-size: ${
                    renameZoom > 1 ? 12 * renameZoom : 12
                  }px; height: ${renameZoom > 1 ? 26 * renameZoom : 26}px;`
                : ''
            "
            :type="nativeInputTagTypeProp"
            :value="modelValue"
            @blur="onBlur"
            @focus="onFocus"
            @input="onChange"
            @keydown="onKeyboardEvent"
          />
        </template>
      </div>

      <div v-if="!compact" class="vorm-input__instructions">
        <slot name="instructions">{{ instructions }}</slot>
      </div>

      <div
        v-if="isError"
        :class="
          clsx(
            compact && [
              'border-b',
              isFocus ? 'border-transparent' : 'border-destructive-600',
            ],
          )
        "
      >
        <div v-if="validationState.isError" class="vorm-input__error-message">
          {{ validationState.errorMessage }}
        </div>
        <div v-if="error_set" class="vorm-input__error-message">
          {{ error_set }}
        </div>
      </div>
    </div>
    <slot name="rightOfInput"></slot>
  </div>
</template>

<script lang="ts" setup>
/* eslint-disable @typescript-eslint/no-explicit-any,@typescript-eslint/no-non-null-assertion */

import { ref, computed, onMounted, onUpdated, watch, toRefs, toRaw } from "vue";
import * as _ from "lodash-es";

import clsx from "clsx";
import Icon from "../icons/Icon.vue";
import { IconNames } from "../icons/icon_set";

import { useValidatedInput, validators } from "./helpers/form-validation";
import { useDisabledBySelfOrParent } from "./helpers/form-disabling";
import VormInputOption from "./VormInputOption.vue";
import { themeClasses, useTheme } from "../utils/theme_tools";
import type { PropType, ComponentInternalInstance } from "vue";

type InputTypes =
  | "checkbox"
  | "container"
  | "date"
  | "decimal"
  | "dropdown"
  | "dropdown-optgroup"
  | "email"
  | "integer"
  | "money"
  | "multi-checkbox"
  | "number"
  | "password"
  | "percent"
  | "radio"
  | "slug"
  | "tel"
  | "text"
  | "textarea"
  | "time-string"
  | "url";

// object of the shape { option1Value: 'Label 1',  }
type OptionsAsSimpleObject = Record<string, string>;
type OptionsAsObjectWithLabels = Record<
  string,
  { label: string; [key: string]: any }
>;
type OptionsAsSimpleArray = string[];
export type OptionsAsArray = { value: ModelValue; label: string }[];
type OptionsAsNestedArrays = Record<string, OptionsAsArray>;
export type InputOptions =
  | OptionsAsSimpleObject
  | OptionsAsObjectWithLabels
  | OptionsAsSimpleArray
  | OptionsAsArray
  | OptionsAsNestedArrays;

type ModelValue = string | number | string[] | boolean | null;
const props = defineProps({
  // to be used with v-model
  // TODO: make modelValue more flexible and typed better
  modelValue: {
    // TODO: resolve some ts issues around binding modelValue to
    type: [String, Number, Array, Boolean, null] as PropType<ModelValue>,
  },
  type: { type: String as PropType<InputTypes>, default: "text" },
  id: { type: String, required: false },

  size: { type: String as PropType<"xs" | "sm" | "md">, default: "md" },

  // label, placeholder, additional text instructions
  label: { type: String },
  prompt: { type: String },
  noLabel: { type: Boolean },
  inlineLabel: { type: Boolean },
  instructions: String,
  placeholder: String,
  labelTooltip: String, // Only works for compact!

  iconRight: { type: String as PropType<IconNames>, required: false },
  iconRightRotate: {
    type: String as PropType<"left" | "right" | "up" | "down">,
    default: undefined,
  },
  nullLabel: String,

  disabled: Boolean,
  defaultValue: [String, Number, Array, Boolean, null] as PropType<ModelValue>,
  autocomplete: { type: String },
  name: { type: String },
  showCautionLines: Boolean,

  // validations
  required: Boolean,
  requiredWarning: Boolean,
  requiredMessage: { type: String, default: "This field is required" },
  min: [Date, Number],
  max: [Date, Number],
  minLength: Number,
  maxLength: Number,
  toUpperCase: Boolean,
  toLowerCase: Boolean,
  digitsOnly: Boolean,
  regex: { type: [String, RegExp] },
  regexMessage: { type: String, default: "This field is invalid" },

  // additional options for specific input types
  // TODO: figure out how to further constrain sets of props to get better autocomplete

  // for radio / dropdown / multi-checkbox / dropdown-optgroup
  options: { type: [Object, Array] as PropType<InputOptions> },

  // for dropdown only
  autoSelect: Boolean,
  placeholderSelectable: Boolean,

  // for checkbox/toggle only
  checkedValue: {
    type: [String, Boolean, Number, null] as PropType<
      string | boolean | number | null
    >,
    default: true,
  },
  uncheckedValue: {
    type: [String, Boolean, Number, null] as PropType<
      string | boolean | number | null
    >,
    default: null,
  },
  reverseCheckedValue: Boolean,

  // for password only
  allowShowPassword: Boolean,
  checkPasswordStrength: Boolean,

  // new compact styling for VormInput
  compact: Boolean,
  compactWide: Boolean,

  // remove all styles from VormInput and fill available area, only works for text input fields
  noStyles: Boolean,

  // special styles for renaming on the ModelingDiagram
  rename: Boolean,
  renameZoom: { type: Number, default: 1 },
});

const wrapperRef = ref<HTMLDivElement>(); // template ref
const inputRef = ref<HTMLInputElement>(); // template ref

const emit = defineEmits([
  "update:modelValue",
  "focus",
  "blur",
  "enterPressed",
]);

// const originalValue = ref(props.modelValue); // store the original value
const { modelValue: currentValue, disabled } = toRefs(props);
const isFocus = ref(false);

const disabledBySelfOrParent = useDisabledBySelfOrParent(disabled);

const isPasswordMasked = ref(true); // only relevant for password

// TODO - add html attributes which are dynamically generated for password attrs

const validationRules = computed(() => {
  const rules = [];
  if (props.required || props.requiredWarning) {
    rules.push({
      fn:
        props.type === "checkbox"
          ? validators.equals(props.checkedValue)
          : validators.required,
      message: props.requiredMessage,
      ...(props.requiredWarning && { warning: true }),
    });
  }
  if (props.type === "url")
    rules.push({ fn: validators.url, message: "Invalid URL" });
  if (props.type === "email")
    rules.push({ fn: validators.email, message: "Invalid email" });
  if (props.type === "time-string")
    rules.push({ fn: validators.timeString, message: "Invalid time string" });

  if (props.regex) {
    rules.push({
      fn: validators.regex(props.regex),
      message: props.regexMessage,
    });
  }
  if (props.minLength) {
    rules.push({
      fn: validators.minLength(props.minLength),
      message: `Your ${
        props.type === "password" ? "password" : "input"
      } must be at least ${props.minLength} characters long.`,
    });
  }
  if (props.maxLength) {
    rules.push({
      fn: validators.maxLength(props.maxLength),
      message: `Your ${
        props.type === "password" ? "password" : "input"
      } cannot be more than ${props.maxLength} characters long.`,
    });
  }
  // TODO: add more rule checks
  // add rules for password strength validation

  return rules;
});

const { validationState, validationMethods } = useValidatedInput(
  currentValue!,
  validationRules,
);

// textarea typescript wont bind to null, so we have this annoying workaround
// TODO: ideally we can just override the TS type for v-model on a textarea somehow...
const modelValueForTextArea = ref("");
watch(
  () => props.modelValue,
  () => {
    modelValueForTextArea.value = props.modelValue?.toString() || "";
  },
  { immediate: true },
);

const { theme } = useTheme();

const isError = computed(() => validationState.isError || error_set.value);

const computedClasses = computed(() => ({
  "--error": isError.value,
  "--focused": isFocus.value,
  "--disabled": disabledBySelfOrParent.value,
  [`--type-${props.type}`]: true,
  [`--size-${props.size}`]: true,
  [`--theme-${theme.value}`]: true,
}));

// shared counter to generate unique IDs used for label + input tag binding
// TODO: probably need to deal with component reuse issues to reset this?

const formInputId = props.id || _.uniqueId("vorm-input-");

const TYPES_WITH_OPTIONS = [
  "radio",
  "dropdown",
  "multi-checkbox",
  "dropdown-optgroup",
];
const NUMERIC_TYPES = ["number", "integer", "decimal", "money", "percent"];
const isTypeWithOptions = computed(() =>
  TYPES_WITH_OPTIONS.includes(props.type),
);

// native html input tag type - ex: <input type="?">
// not used when the tag is not an input (textarea, select)
const nativeInputTagTypeProp = computed(() => {
  if (props.type === "textarea") return undefined;
  if (props.type === "password" && isPasswordMasked.value) return "password";
  if (NUMERIC_TYPES.includes(props.type)) return "number";
  return "text";
});
const nativeInputNumberStepProp = computed(() => {
  if (
    props.type === "decimal" ||
    props.type === "money" ||
    props.type === "percent"
  )
    return 0.01;
  if (props.type === "integer" || props.type === "number") return 1;
  return undefined;
});

const computedPlaceholder = computed(() => {
  if (props.placeholder) return props.placeholder;
  if (props.type === "date") return "YYYY-MM-DD";
  return undefined;
});

function cleanValue(inputVal: ModelValue): ModelValue | null {
  const val: any = inputVal;
  // called when setting a new value to clean/coerce values
  if (val === "") return null;
  if (!val) return val;

  // text types ///////////////////////////////////////////////////////////
  if (props.type === "url") {
    return `${val.startsWith("http") ? "" : "https://"}${val}`;
  } else if (props.type === "email") {
    return val.trim().toLowerCase();
  } else if (props.type === "text") {
    let cleanVal = val.toString().trim();
    if (props.digitsOnly) cleanVal = cleanVal.replace(/[^0-9]/g, "");
    if (props.toLowerCase) cleanVal = cleanVal.toLowerCase();
    if (props.toUpperCase) cleanVal = cleanVal.toUpperCase();
    return cleanVal;
  } else if (props.type === "tel") {
    return (val as string).replace(/[^0-9+]/g, "");
  } else if (props.type === "date") {
    try {
      return new Date(val as number | string).toISOString().slice(0, 10);
    } catch (err) {
      return val;
    }

    // numeric types ///////////////////////////////////////////////////////////
  } else if (props.type === "number") {
    // default "number" behaviour pins min to 0 and rounds to nearest int
    return Math.max(0, Math.round(val));
  } else if (props.type === "integer") {
    return Math.round(val);
  } else if (props.type === "decimal" || props.type === "money") {
    return parseFloat(val);
  }

  return val;
}

function setNewValue(newValue: ModelValue, clean = true) {
  emit("update:modelValue", clean ? cleanValue(newValue) : newValue);
}

// helpers to deal with input types that have child options (radio, multicheckbox, dropdown, dropdown-optgroup)
function generateOptionKey(option: { value: ModelValue }, index: number) {
  return `vorm-input-option-${formInputId}-${index}`;
}

const arrayOptionsFromProps = computed((): OptionsAsArray => {
  /* eslint-disable consistent-return */
  if (!isTypeWithOptions.value) return [];
  if (!props.options) return [];
  if (props.type === "dropdown-optgroup") {
    return Object.values(toRaw(props.options) as OptionsAsNestedArrays).flat();
  } else if (_.isArray(props.options)) {
    if (!_.isObject(props.options[0])) {
      return _.map(props.options as OptionsAsSimpleArray, (value) => ({
        value,
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        label: value.toString(),
      }));
    }
    // handle array of simple strings
    if (_.isString(props.options[0])) {
      return _.map(props.options as OptionsAsSimpleArray, (value) => ({ value, label: value }));
    }
    // otherwise its an array of { value, label } and we can pass through as is
    return props.options as OptionsAsArray;
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

// TODO(nick): make this safe for other options coming in. I think the bigger problem is that how
// the props work in general though...
const objectOptionsFromProps = computed((): OptionsAsNestedArrays => {
  if (!isTypeWithOptions.value) return {};
  if (!props.options) return {};
  if (_.isArray(props.options)) return {};
  if (!_.isObject(props.options)) return {};
  return props.options as OptionsAsNestedArrays;
});

// event handlers
function onFocus(evt: Event) {
  isFocus.value = true;
  emit("focus", evt);
}

function onBlur() {
  isFocus.value = false;
  // inputs with child options fire change events from the VormInputOption component
  if (!isTypeWithOptions.value && props.modelValue !== undefined) {
    emit("update:modelValue", cleanValue(props.modelValue));
  }
  validationMethods.touch();
  emit("blur");
}

function onChange(event: Event) {
  if (
    event.target instanceof HTMLInputElement ||
    event.target instanceof HTMLTextAreaElement
  ) {
    // do not run "cleaning" logic on change - it will be cleaned on blur
    setNewValue(event.target?.value, false);
  }
}

function onKeyboardEvent(event: KeyboardEvent) {
  const key = event.key;
  if (key === "Enter") {
    inputRef.value?.blur();
    emit("enterPressed");
  }

  if (NUMERIC_TYPES.includes(props.type)) {
    // prevent typing more than one "."
    if (key === "." && (props.modelValue as any)?.toString().includes(".")) {
      event.preventDefault();
    }

    // prevent typing e/E/+ since they can be valid numbers in default html tag
    if (["e", "E", "+"].includes(key)) {
      event.preventDefault();
    }
  }
}

const childInputOptions = ref([] as ComponentInternalInstance[]);

function registerChildInputOption(
  inputOptionComponent: ComponentInternalInstance,
) {
  childInputOptions.value.push(inputOptionComponent as any);
}

function unregisterChildInputOption(
  inputOptionComponent: ComponentInternalInstance,
) {
  childInputOptions.value = _.reject(childInputOptions.value, {
    uid: inputOptionComponent.uid,
  });
}

// some specific type handlers

// DROPDOWN + MULTI CHECKBOX TYPE ///////////////////////////////////////////////////////////////////////////////////////
function onSelectChange(event: Event) {
  // TODO: a little extra handling to grab the actual vue child and use its bound value
  // rather than event.target.value as this will allow us to preserve any weird value types

  const newSelectedValue = [...(event?.target as any).selectedOptions].pop()
    ?.value;
  // fallback to event.target.value for cases where the VormInputOption has no bound value
  // for example `VormInputOption yes`

  // console.log(childIndex, selectedOption, newSelectedValue);
  const newVal =
    newSelectedValue === undefined
      ? (event?.target as any)?.value
      : newSelectedValue;
  setNewValue(newVal);
}

function fixOptionSelection() {
  // if currently selected value doesnt exist reset selection to null
  const possibleChildValues = _.map(childInputOptions.value, (input) => {
    return input?.exposed?.optionValue;
  });

  // first deal with keeping only valid values
  if (props.type === "multi-checkbox") {
    // only keep valid values
    const validValues = _.intersection(
      props.modelValue as string[],
      possibleChildValues,
    );
    // TODO figure out how we want to deal with defaulting empty to `null` vs `[]`
    if (!_.isEqual(validValues, props.modelValue)) setNewValue(validValues);
  } else if (!possibleChildValues.includes(props.modelValue)) {
    setNewValue(null);
  }

  // now deal with auto-select (ie automatically select the first option)
  // which happens on load, but could also be after the selected option was removed from the list
  if (props.autoSelect && _.isNil(props.modelValue)) {
    let autoSelectIndex = 0;
    // dropdown has an actual child option as the placeholder
    if (props.type === "dropdown" && props.placeholder) autoSelectIndex = 1;

    const autoSelectOptionComponent = childInputOptions.value[autoSelectIndex];
    if (autoSelectOptionComponent)
      setNewValue(autoSelectOptionComponent?.exposed?.optionValue);
  }
}

// have to do some special handling for selects that are using empty and boolean values
// see VormInputOption for more info
const valueForSelectField = computed(() => {
  if (typeof props.modelValue === "boolean") return String(props.modelValue);
  if (props.modelValue === undefined) return props.nullLabel ?? "_null_";
  if (props.modelValue === null) return props.nullLabel ?? "_null_";

  return props.modelValue;
});

// CHECKBOX TYPE ////////////////////////////////////////////////////////////////////////////////////////
function onCheckboxChange(event: Event) {
  const checked = (event.target as HTMLInputElement).checked;
  setNewValue(checked ? props.checkedValue : props.uncheckedValue);
}

// some extra handling to fix the selected value when options change
// also handles selecting the first
onMounted(() => {
  // set default value if value is empty and defaultValue prop is provided
  if (_.isNil(props.modelValue) && props.defaultValue)
    setNewValue(props.defaultValue);

  if (isTypeWithOptions.value) fixOptionSelection();
});
onUpdated(() => {
  if (isTypeWithOptions.value) fixOptionSelection();
});

// also can be useful to expose a focus method if other things need to trigger focus programatically
function focus() {
  if (["multi-checkbox", "radio"].includes(props.type)) {
    const inputEls = wrapperRef?.value?.getElementsByTagName("input");
    inputEls?.item(0)?.focus();
  } else {
    if (inputRef.value?.focus)
      inputRef.value.focus({ preventScroll: props.rename });
  }
}

function blur() {
  if (["multi-checkbox", "radio"].includes(props.type)) {
    const inputEls = wrapperRef?.value?.getElementsByTagName("input");
    inputEls?.item(0)?.blur();
  } else {
    if (inputRef.value?.blur) inputRef.value.blur();
  }
}

const error_set = ref<string | null>(null);
function setError(msg: string) {
  error_set.value = msg;
}

defineExpose({
  focus,
  blur,
  isFocus,

  // expose some props for child `VormInputOption` instances to use
  // as these components are tightly coupled and meant to be used together
  // TODO: we could pass these down as props or use events...
  vormInputType: props.type,
  formInputId,
  currentValue,
  onChildInputOptionBlur: onBlur,
  onChildInputOptionFocus: onFocus,
  onChildInputOptionChange: setNewValue,
  registerChildInputOption,
  unregisterChildInputOption,

  // TODO: hopefully vue will start merging exposed properties?
  // ideally these should already be exposed from calling `useValidatedInput()`
  // but this second call to `expose` was overwriting that one, so we must include it again
  validationState,
  validationMethods,
  setError,

  // Just in case you need to access the DOM element for the input field
  inputRef,
});
</script>

<style lang="less" scoped>
// TODO: probably remove all these colors and use tw classes?

@vertical-gap: 8px;

.vorm-input-standard {
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
    .vorm-input__input {
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

    input,
    select,
    textarea {
      cursor: not-allowed;
      color: currentColor;
    }
  }

  .vorm-input__instructions,
  .vorm-input__error-message {
    @apply capsize text-xs;
    padding-top: @vertical-gap;
    padding-bottom: 4px;
  }
}

// this class is on whatever the input is, whether its input, textarea, select, etc
.vorm-input__input {
  width: 100%;
  border: 1px solid var(--border-color);
  border-radius: 3px;
  transition: border-color 0.15s;
  padding: 4px 12px;
  color: var(--text-color);
  font: inherit;
  background-color: var(--bg-color);

  .vorm-input.--size-xs & {
    padding: 2px 10px;
    height: 32px;
  }
  .vorm-input.--size-sm & {
    padding: 4px 10px;
    height: 32px;
  }
  .vorm-input.--size-md & {
    padding: 8px 12px;
  }

  &:hover {
    --border-color: @colors-neutral-500;
  }

  textarea& {
    min-height: 80px;
    display: block;
  }

  select& {
    -webkit-appearance: none;
    padding-right: 28px !important; // to make space for dropdown arrow

    // dropdown arrow on right
    background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='40' height='40' fill='%23666'><polygon points='10,15 30,15 20,25'/></svg>");
    background-size: 25px 25px;
    background-position: calc(100% - 4px) center;
    background-repeat: no-repeat;

    &.--placeholder-selected {
      color: var(--text-color-muted);
      font-style: italic;

      &.--theme-dark {
        color: @colors-neutral-500;
      }
    }

    &:-moz-focusring {
      color: transparent;
      text-shadow: 0 0 0 #000;
    }
  }

  // set font size for our inputs
  // input[type='text']&,
  // input[type='number']&,
  // input[type='password']&,
  input&,
  select&,
  textarea& {
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
}

.vorm-input__pass-show-hide-toggle + .vorm-input__input {
  padding-right: 35px;
}

.vorm-input-standard > .vorm-input__label {
  @apply capsize text-sm;
  align-items: center;
  color: currentColor;
  font-weight: 700;
  padding-left: 1px;

  > .icon {
    margin-right: 4px;
    height: 14px;
    width: 14px;
  }
}

.vorm-input__locked-icon {
  margin-right: 4px;
  width: 12px;
  height: 12px;
  display: inline-block;
  vertical-align: middle;
  margin-top: -1px;
}

.vorm-input__right-icon {
  margin-right: 4px;
  display: inline-block;
  vertical-align: middle;
  margin-top: -1px;
}

.vorm-input__instructions {
  color: var(--text-color-muted);

  a {
    color: currentColor;
    text-decoration: underline;
  }

  .icon {
    height: 16px;
    width: 16px;
    vertical-align: bottom;
    margin-right: 4px;
  }

  &:empty {
    display: none;
  }
}

.vorm-input__prompt {
  &:empty {
    display: none;
  }
}

.vorm-input__input-wrap {
  position: relative;
}

.vorm-input__pass-show-hide-toggle {
  cursor: pointer;
  user-select: none;
  opacity: 0.8;
  font-size: 12px;
  line-height: 40px;
  position: absolute;
  top: 0;
  right: 0;
  margin-right: 8px;
  margin-top: 8px;
  opacity: 0.6;
  z-index: 100;

  &:hover {
    opacity: 1;
  }
}

// small adjustments for specific input types
.vorm-input.--type-radio {
  .vorm-input__input-wrap {
    padding-left: 1px;
  }
}

.vorm-input.--type-checkbox {
  .vorm-input__input-wrap {
    display: flex;
    align-items: center;
  }

  .vorm-input__input {
    display: block;
    width: 24px;
    height: 24px;
    margin-right: 12px;
    flex-shrink: 0;
    cursor: pointer;

    // tailwind base classes are messing with this majorly, so have to reset it a bit
    background-color: var(--bg-color);
    border-color: var(--border-color);

    // TODO: rework how checkboxes are set up... this is because tailwind inserts a background image
    // so below we copied their SVG but just changed the bg colors

    // we probably want to just put an actual Icon(name="check") in the markup instead...
    &:checked {
      background-image: url("data:image/svg+xml,%3csvg viewBox='0 0 16 16' fill='black' xmlns='http://www.w3.org/2000/svg'%3e%3cpath d='M12.207 4.793a1 1 0 010 1.414l-5 5a1 1 0 01-1.414 0l-2-2a1 1 0 011.414-1.414L6.5 9.086l4.293-4.293a1 1 0 011.414 0z'/%3e%3c/svg%3e");
    }
  }

  &.--theme-dark {
    .vorm-input__input {
      &:checked {
        background-image: url("data:image/svg+xml,%3csvg viewBox='0 0 16 16' fill='white' xmlns='http://www.w3.org/2000/svg'%3e%3cpath d='M12.207 4.793a1 1 0 010 1.414l-5 5a1 1 0 01-1.414 0l-2-2a1 1 0 011.414-1.414L6.5 9.086l4.293-4.293a1 1 0 011.414 0z'/%3e%3c/svg%3e");
      }
    }
  }

  .vorm-input__checkbox-text {
    cursor: pointer;
    // padding-top: 5px;
    a {
      // TODO: probably want to set this more broadly somewhere else
      text-decoration: underline;
    }
  }
}

// COMPACT STYLES

.vorm-input-compact {
  position: relative;
  font-size: 14px;
  position: relative;
  display: flex;
  align-items: center;
  width: 100%;

  body.light & {
    --header-bg-color: @colors-neutral-500;
    --header-text-color: @colors-white;
    &.--section-hover {
      --header-bg-color: @colors-neutral-900;
      --header-text-color: @colors-white;
    }
  }
  body.dark & {
    --header-bg-color: @colors-neutral-600;
    --header-text-color: @colors-white;
    &.--section-hover {
      --header-bg-color: @colors-neutral-300;
      --header-text-color: @colors-black;
    }
  }

  .vorm-input__instructions,
  .vorm-input__error-message {
    font-size: 12px;
    padding-top: 2px;
    padding-bottom: 2px;
    color: @colors-destructive-600;
  }
}

.vorm-input-compact > .vorm-input__label {
  cursor: default;
  flex-shrink: 1;
  flex-grow: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  padding: 4px 0; // fixes cut off descenders
  i {
    font-style: normal;
    opacity: 0.5;
  }
}

.vorm-input-compact > .vorm-input__input-and-instructions-wrap {
  position: relative;
  font-family: monospace;
  font-size: 13px;
  line-height: 18px;

  .vorm-input-compact__input__rename {
    font-family: "Inter";
    padding: 4px 4px;
  }

  input,
  textarea {
    background: transparent;
    font-family: inherit;
    padding: 5px 8px;
    width: 100%;
    border: none;
    font-size: inherit;
    line-height: inherit;
    display: block;
    text-overflow: ellipsis;
    overflow: hidden;
    overflow-y: auto;

    .attributes-panel-item.--input.--focus &,
    .attributes-panel-item.--input.--hover & {
      padding-right: 28px; // to give space for unset button
    }
  }
  textarea {
    min-height: 80px;
    margin: 0;
  }

  // chrome + linux showing white on white text - this might fix it?
  select {
    option {
      background: white;
      color: black;
    }
  }

  .attributes-panel-item__type-icon {
    position: absolute;
    left: 0px;
    top: 0px;
    width: 28px;
    height: 28px;
    padding: 3px;
    z-index: 2;
    pointer-events: none;
  }
}

// inputs next to each other push together to overlap their input borders
.vorm-input-compact + .vorm-input-compact {
  margin-top: -1px;
}

.vorm-input-compact__input {
  background-color: var(--bg-color);

  select& {
    width: 100%;
    appearance: none;
    -webkit-appearance: none;
    padding-right: 28px !important; // to make space for dropdown arrow

    // dropdown arrow on right
    background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='40' height='40' fill='%23666'><polygon points='10,15 30,15 20,25'/></svg>");
    background-size: 25px 25px;
    background-position: calc(100% - 4px) center;
    background-repeat: no-repeat;

    &.--placeholder-selected {
      color: var(--text-color-muted);
      font-style: italic;

      &.--theme-dark {
        color: @colors-neutral-500;
      }
    }

    &:-moz-focusring {
      color: transparent;
      text-shadow: 0 0 0 #000;
    }
  }

  // set font size for our inputs
  // input[type='text']&,
  // input[type='number']&,
  // input[type='password']&,
  input&,
  select&,
  textarea& {
    line-height: 1rem;
    font-size: 14px;

    // if font-size is at least 16 on mobile, ios will not automatically zoom in
    @media @mq-mobile-only {
      font-size: 16px;
    }
  }

  textarea& {
    min-height: 80px;
    display: block;
  }
}

.vorm-input__hidden-input {
  position: absolute;
  left: 0;
  right: 0;
  top: 0;
  padding: 0;
  height: 100%;
  opacity: 0;
  z-index: 1;
  display: block;
  cursor: pointer;
}

.vorm-input__input-value {
  padding: 5px 24px 5px 8px;
  display: block;
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
}
</style>
