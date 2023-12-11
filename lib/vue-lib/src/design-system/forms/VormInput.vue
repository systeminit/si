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
    class="vorm-input"
    :class="
      clsx(
        computedClasses,
        inlineLabel ? 'flex flex-row gap-xs items-center' : 'block',
      )
    "
  >
    <label
      v-if="!noLabel"
      :class="clsx('vorm-input__label', !inlineLabel && 'pb-xs block')"
      :for="formInputId"
    >
      <Icon
        v-if="disabledBySelfOrParent"
        :class="clsx('vorm-input__locked-icon')"
        name="lock"
      />
      <slot name="label">
        {{ label || "&nbsp;" }}{{ required || requiredWarning ? "*" : "" }}
      </slot>
    </label>
    <slot name="prompt">
      <div class="vorm-input__prompt pb-xs text-sm">{{ prompt }}</div>
    </slot>
    <div class="vorm-input__input-and-instructions-wrap">
      <div class="vorm-input__input-wrap">
        <template v-if="type === 'container'">
          <slot />
        </template>

        <template v-else-if="type === 'dropdown'">
          <select
            :id="formInputId"
            ref="inputRef"
            class="vorm-input__input"
            :class="[
              modelValue === null && !placeholderSelectable
                ? '--placeholder-selected'
                : '',
            ]"
            :disabled="disabledBySelfOrParent"
            :value="valueForSelectField"
            @focus="onFocus"
            @blur="onBlur"
            @change="onSelectChange"
          >
            <VormInputOption
              v-if="placeholder"
              :value="null"
              :hidden="!placeholderSelectable"
              :disabled="!placeholderSelectable"
            >
              {{ placeholder }}
            </VormInputOption>

            <slot />

            <VormInputOption
              v-for="(o, i) in optionsFromProps"
              :key="generateOptionKey(o, i)"
              :value="o.value"
              >{{ o.label }}
            </VormInputOption>
          </select>
        </template>

        <template v-else-if="type === 'radio' || type === 'multi-checkbox'">
          <VormInputOption
            v-for="(o, i) in optionsFromProps"
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
            class="vorm-input__input"
            :checked="modelValue === checkedValue"
            type="checkbox"
            :disabled="disabledBySelfOrParent"
            @input="onCheckboxChange"
          />
          <label class="vorm-input__checkbox-text" :for="formInputId">
            <slot />
          </label>
        </template>

        <template v-else-if="type === 'textarea'">
          <textarea
            :id="formInputId"
            ref="inputRef"
            :value="modelValueForTextArea"
            class="vorm-input__input"
            :placeholder="computedPlaceholder"
            :disabled="disabledBySelfOrParent"
            :maxlength="maxLength"
            @focus="onFocus"
            @blur="onBlur"
            @input="onChange"
          />
        </template>

        <template v-else>
          <Icon
            v-if="type === 'password' && allowShowPassword"
            class="vorm-input__pass-show-hide-toggle"
            :name="isPasswordMasked ? 'show' : 'hide'"
            allow-pointer-events
            @click="isPasswordMasked = !isPasswordMasked"
          />
          <input
            :id="formInputId"
            ref="inputRef"
            :value="modelValue"
            class="vorm-input__input"
            :autocomplete="autocomplete"
            :name="name"
            :type="nativeInputTagTypeProp"
            :placeholder="computedPlaceholder"
            :disabled="disabledBySelfOrParent"
            :step.prop="nativeInputNumberStepProp"
            :minlength="minLength"
            :maxlength="maxLength"
            :passwordrules="
              type === 'password'
                ? `minlength: ${minLength}; maxlength: ${maxLength}; required: lower; required: upper; required: digit; required: special;`
                : undefined
            "
            @keydown="onKeyboardEvent"
            @focus="onFocus"
            @blur="onBlur"
            @input="onChange"
          />
        </template>
      </div>

      <div class="vorm-input__instructions">
        <slot name="instructions">{{ instructions }}</slot>
      </div>

      <div v-if="validationState.isError" class="vorm-input__error-message">
        {{ validationState.errorMessage }}
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
/* eslint-disable @typescript-eslint/no-explicit-any,@typescript-eslint/no-non-null-assertion */

import { ref, computed, onMounted, onUpdated, watch, toRefs } from "vue";
import * as _ from "lodash-es";

import clsx from "clsx";
import Icon from "../icons/Icon.vue";

import { useValidatedInput, validators } from "./helpers/form-validation";
import { useDisabledBySelfOrParent } from "./helpers/form-disabling";
import VormInputOption from "./VormInputOption.vue";
import { useTheme } from "../utils/theme_tools";
import type { PropType, ComponentInternalInstance } from "vue";

type InputTypes =
  | "container"
  | "text"
  | "textarea"
  | "email"
  | "url"
  | "password"
  | "slug"
  | "tel"
  | "number"
  | "integer"
  | "decimal"
  | "money"
  | "percent"
  | "date"
  | "checkbox"
  | "multi-checkbox"
  | "dropdown"
  | "radio";

// object of the shape { option1Value: 'Label 1',  }
type OptionsAsSimpleObject = Record<string, string>;
type OptionsAsObjectWithLabels = Record<
  string,
  { label: string; [key: string]: any }
>;
type OptionsAsSimpleArray = string[];
type OptionsAsArray = { value: any; label: string }[];
type InputOptions =
  | OptionsAsSimpleObject
  | OptionsAsObjectWithLabels
  | OptionsAsSimpleArray
  | OptionsAsArray;

const props = defineProps({
  // to be used with v-model
  // TODO: make modelValue more flexible and typed better
  modelValue: {
    // TODO: resolve some ts issues around binding modelValue to
    type: [String, Number, Array, Boolean, null] as PropType<
      string | number | string[] | boolean | null
    >,
    // type: [String, Number, Array, Boolean, null] as PropType<any>,
  },
  type: { type: String as PropType<InputTypes>, default: "text" },

  size: { type: String as PropType<"xs" | "sm" | "md">, default: "md" },

  // label, placeholder, additional text instructions
  label: { type: String },
  prompt: { type: String },
  noLabel: { type: Boolean },
  inlineLabel: { type: Boolean },
  instructions: String,
  placeholder: String,

  disabled: Boolean,
  defaultValue: {}, // eslint-disable-line vue/require-prop-types
  autocomplete: { type: String },
  name: { type: String },

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

  // for radio / dropdown / multi-checkbox
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

const computedClasses = computed(() => ({
  "--error": validationState.isError,
  "--focused": isFocus.value,
  "--disabled": disabledBySelfOrParent.value,
  [`--type-${props.type}`]: true,
  [`--size-${props.size}`]: true,
  [`--theme-${theme.value}`]: true,
}));

// shared counter to generate unique IDs used for label + input tag binding
// TODO: probably need to deal with component reuse issues to reset this?

const formInputId = _.uniqueId("vorm-input-");

const TYPES_WITH_OPTIONS = ["radio", "dropdown", "multi-checkbox"];
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

function cleanValue(val: any) {
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
    return val.replace(/[^0-9+]/g, "");
  } else if (props.type === "date") {
    try {
      return new Date(val).toISOString().slice(0, 10);
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

function setNewValue(newValue: any, clean = true) {
  emit("update:modelValue", clean ? cleanValue(newValue) : newValue);
}

// helpers to deal with input types that have child options (radio, multicheckbox, dropdown)
function generateOptionKey(option: { value: string }, index: number) {
  return `vorm-input-option-${formInputId}-${index}`;
}

const optionsFromProps = computed((): OptionsAsArray => {
  /* eslint-disable consistent-return */
  if (!isTypeWithOptions.value) return [];
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
      return _.map(props.options, (value) => ({ value, label: value })) as any;
    }
    // otherwise its an array of { value, label } and we can pass through as is
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

// event handlers
function onFocus() {
  isFocus.value = true;
  emit("focus");
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
    onBlur();
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

  const childIndex = (event?.target as any)?.selectedIndex;
  const selectedOption = childInputOptions.value[childIndex];
  const newSelectedValue = selectedOption?.exposed?.optionValue;
  // fallback to event.target.value for cases where the VormInputOption has no bound value
  // for example `VormInputOption yes`

  // console.log(childIndex, selectedOption, newSelectedValue);
  setNewValue(
    newSelectedValue === undefined
      ? (event?.target as any)?.value
      : newSelectedValue,
  );
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
  if (props.modelValue === undefined) return "_null_";
  if (props.modelValue === null) return "_null_";

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
    if (inputRef.value?.focus) inputRef.value.focus();
  }
}

defineExpose({
  focus,

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

  // Just in case you need to access the DOM element for the input field
  inputRef,
});
</script>

<style lang="less" scoped>
// TODO: probably remove all these colors and use tw classes?

@vertical-gap: 8px;

.vorm-input {
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
      color: white;
      font-style: italic;
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

.vorm-input__pass-show-hide-toggle + .vorm-input__input {
  padding-right: 35px;
}

.vorm-input__label {
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

.vorm-input__instructions,
.vorm-input__error-message {
  @apply capsize text-xs;
  padding-top: @vertical-gap;
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

.vorm-input__error-message {
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
</style>
