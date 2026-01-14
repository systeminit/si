<template>
  <div>
    <div v-for="error in errors" :key="error.id" :class="errorClasses(error.id)" v-bind="$attrs">
      {{ error.message }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, toRefs } from "vue";
import * as _ from "lodash-es";
import { ValidatorArray, ErrorsArray } from "@/utils/input_validations";

const props = defineProps<{
  validations?: ValidatorArray;
  value: string;
}>();
const emit = defineEmits<{ (e: "errors", errors: ErrorsArray): void }>();

const errors = ref<ErrorsArray>([]);

const { value, validations } = toRefs(props);

const evaluateErrors = (newValue: string, validations?: ValidatorArray) => {
  const currentErrors: ErrorsArray = [];

  for (const v of validations ?? []) {
    if (v.check(newValue) === false) {
      currentErrors.push({ id: v.id, message: v.message });
    }
  }

  errors.value = currentErrors;
  emit("errors", errors.value);
};

watch(
  [() => value.value, () => validations?.value],
  ([newValue, newValidations]) => {
    evaluateErrors(newValue, newValidations);
  },
  { immediate: true },
);

const errorClasses = (id: string) => {
  const classes: Record<string, boolean> = {};
  classes["text-xs"] = true;
  classes["lg:text-sm"] = true;
  classes["text-destructive-500"] = true;
  classes["dark:text-destructive-600"] = true;
  return classes;
};
</script>

<script lang="ts">
export default {
  inheritAttrs: false,
};
</script>
