<template>
  <div>
    <div
      v-for="error in displayErrors"
      :key="error.id"
      :class="errorClasses(error.id)"
      v-bind="$attrs"
    >
      {{ error.message }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, toRefs, computed } from "vue";
import _ from "lodash";
import { ValidatorArray, ErrorsArray } from "@/utils/input_validations";

const props = defineProps<{
  required?: boolean;
  showRequired?: boolean;
  validations?: ValidatorArray;
  value: string;
  dirty: boolean;
  hideRequiredUnlessDirty?: boolean;
}>();
const emit = defineEmits<{ (e: "errors", errors: ErrorsArray): void }>();

const errors = ref<ErrorsArray>([]);

const { value, validations, required, showRequired, dirty } = toRefs(props);

const evaluateErrors = (newValue: string, validations?: ValidatorArray) => {
  const currentErrors: ErrorsArray = [];

  if (required?.value && value.value.length === 0) {
    currentErrors.push({
      id: "required",
      message: "This field is required.",
    });
  }

  if (validations) {
    for (const v of validations) {
      if (v.check(newValue) === false) {
        if (dirty.value) {
          currentErrors.push({ id: v.id, message: v.message });
        }
      }
    }
  }

  errors.value = currentErrors;
  emit("errors", errors.value);
};

watch(
  [() => value.value, () => dirty.value, () => validations?.value],
  ([newValue, _newDirty, newValidations]) => {
    if (newValue) {
      evaluateErrors(newValue, newValidations);
    }
  },
  { immediate: true },
);

const displayErrors = computed(() => {
  if (
    props.hideRequiredUnlessDirty &&
    props.dirty &&
    errors.value.length === 1
  ) {
    return errors.value;
  } else if (!showRequired?.value) {
    return _.filter(errors.value, (e) => {
      return e.id !== "required";
    });
  } else {
    return errors.value;
  }
});

const errorClasses = (id: string) => {
  const classes: Record<string, boolean> = {};
  classes["text-xs"] = true;
  classes["lg:text-sm"] = true;
  if (id === "required" && !props.hideRequiredUnlessDirty) {
    classes["text-neutral-500"] = true;
  } else {
    classes["text-destructive-400"] = true;
  }
  return classes;
};
</script>

<script lang="ts">
export default {
  inheritAttrs: false,
};
</script>
