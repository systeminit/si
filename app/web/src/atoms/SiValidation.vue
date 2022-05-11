<template>
  <div
    v-for="error in displayErrors"
    :key="error.id"
    :class="errorClasses(error.id)"
    v-bind="$attrs"
  >
    {{ error.message }}
  </div>
</template>

<script lang="ts">
export default {
  inheritAttrs: false,
};
</script>

<script setup lang="ts">
import { ref, watch, toRefs, computed } from "vue";
import _ from "lodash";

export type ValidatorArray = Array<{
  id: string;
  check: (v: string) => boolean;
  message: string;
}>;
export type ErrorsArray = Array<{ id: string; message: string }>;

const props = defineProps<{
  required?: boolean;
  showRequired?: boolean;
  validations?: ValidatorArray;
  value: string;
  dirty: boolean;
}>();
const emit = defineEmits<{ (e: "errors", errors: ErrorsArray): void }>();

const errors = ref<ErrorsArray>([]);

const evaluateErrors = (newValue: string) => {
  const currentErrors: ErrorsArray = [];

  if (required?.value && value.value.length == 0) {
    currentErrors.push({
      id: "required",
      message: "This field is required.",
    });
  }

  if (validations?.value) {
    for (const v of validations.value) {
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

const { value, validations, required, showRequired, dirty } = toRefs(props);
watch(
  value,
  (newValue, _oldValue) => {
    evaluateErrors(newValue);
  },
  { immediate: true },
);
watch(
  dirty,
  (newValue) => {
    if (newValue) {
      evaluateErrors(value.value);
    }
  },
  { immediate: true },
);

const displayErrors = computed(() => {
  if (!showRequired?.value) {
    return _.filter(errors.value, (e) => {
      return e.id != "required";
    });
  } else {
    return errors.value;
  }
});

const errorClasses = (id: string) => {
  const classes: Record<string, boolean> = {};
  classes["text-xs"] = true;
  classes["lg:text-sm"] = true;
  if (id === "required") {
    classes["text-gray-500"] = true;
  } else {
    classes["text-red-400"] = true;
  }
  return classes;
};
</script>
