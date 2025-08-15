/* This component is tightly coupled with the VormInput component and is only
meant to be used within it specifically it's only for VormInputs with type =
dropdown | radio | multi-checkbox */
<template>
  <option
    v-if="
      parentInputType === 'dropdown' || parentInputType === 'dropdown-optgroup'
    "
    class="vorm-input-option"
    :value="safeOptionValue"
    :selected="dropdownOptionSelected"
    :disabled="disabledBySelfOrParent"
  >
    <slot>{{ value }}</slot>
  </option>
  <label v-else-if="parentInputType === 'radio'" class="vorm-input-option">
    <input
      class="vorm-input-option__input"
      type="radio"
      :value="safeOptionValue"
      :name="radioName"
      :disabled="disabledBySelfOrParent"
      :checked="value === parentValue"
      @focus="onFocus"
      @blur="onBlur"
      @change="onChange"
    />
    <div>
      <slot>{{ value }}</slot>
    </div>
  </label>

  <label
    v-else-if="parentInputType === 'multi-checkbox'"
    class="vorm-input-option"
  >
    <input
      class="vorm-input-option__input"
      type="checkbox"
      :value="safeOptionValue"
      :disabled="disabledBySelfOrParent"
      :checked="parentValue?.indexOf(value) > -1"
      @focus="onFocus"
      @blur="onBlur"
      @change="onMultiCheckboxChange"
    />
    <div>
      <slot>{{ value }}</slot>
    </div>
  </label>
</template>

<script lang="ts" setup>
import {
  computed,
  onMounted,
  toRefs,
  getCurrentInstance,
  onBeforeUnmount,
  toRaw,
  toRef,
  unref,
} from "vue";
import * as _ from "lodash-es";

import { useDisabledBySelfOrParent } from "./helpers/form-disabling";
import type { PropType } from "vue";

const props = defineProps({
  value: [String, Number, Array, Boolean] as PropType<
    string | number | string[] | boolean | null
  >,
  disabled: Boolean,
});

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const vormInputParent = getCurrentInstance()?.parent as any;

// NOTE - having some issues with vue deciding when to reuse inputs from within Stack
// which was causing this exposed data to be incorrect... instead using the props directly seems to fix it

// const parentInputType = vormInputParent?.exposed?.vormInputType; // this causes some issues in very specific cases
const parentInputType = vormInputParent.props.type; // this seems to work fine

const parentValue = computed(() =>
  unref(vormInputParent?.exposed?.currentValue),
);

// this is tightly coupled component only meant to be used within a VormInput
const VALID_PARENT_INPUT_TYPES = [
  "dropdown",
  "radio",
  "multi-checkbox",
  "dropdown-optgroup",
];
if (!VALID_PARENT_INPUT_TYPES.includes(parentInputType)) {
  throw new Error(
    "VormInputOption can only be used inside VormInput with type = dropdown/radio/multi-checkbox",
  );
}

const { disabled } = toRefs(props);
const disabledBySelfOrParent = useDisabledBySelfOrParent(disabled);

// set value used for select (dropdown) options - which have some extra handling for empty/null
// because otherwise it will default to the text content
const safeOptionValue = computed(() => {
  if (parentInputType === "dropdown") {
    if (props.value === undefined) return "_null_";
    if (props.value === null) return "_null_";
    return props.value?.toString();
  }
  return props.value;
});

const dropdownOptionSelected = computed(
  () => parentInputType === "dropdown" && props.value === parentValue.value,
);

const radioName = vormInputParent?.exposed.formInputId;

function onBlur() {
  vormInputParent?.exposed.onChildInputOptionBlur();
}
function onFocus() {
  vormInputParent?.exposed.onChildInputOptionFocus();
}
function onChange() {
  // use props.value instead of the native js event target value
  // to avoid any other transformation (converting to string)
  vormInputParent?.exposed.onChildInputOptionChange(props.value);
}
function onMultiCheckboxChange() {
  const currentValue = toRaw(parentValue.value);
  let newValue; // emit new array rather than modifying so reactive change is noticed
  if (_.includes(currentValue, props.value)) {
    newValue = _.without(currentValue || [], props.value);
  } else {
    newValue = [...(currentValue || []), props.value];
  }
  vormInputParent?.exposed.onChildInputOptionChange(
    newValue.length ? newValue : null,
  );
}

// register each option with parent input component
const thisComponent = getCurrentInstance();
onMounted(() => {
  vormInputParent?.exposed.registerChildInputOption(thisComponent);
});
onBeforeUnmount(() => {
  vormInputParent?.exposed.unregisterChildInputOption(thisComponent);
});

// expose the value so the parent can access it

defineExpose({
  optionValue: toRef(props, "value"), // have to expose a ref or value gets stuck if the component is reused
});
</script>

<style lang="less">
label.vorm-input-option {
  display: flex;
  flex-direction: row;
  align-items: center;
  padding-bottom: 0.25rem;
}
.vorm-input-option__input {
  margin-right: 8px;
}
</style>
