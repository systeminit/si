import { ref, computed } from "vue";
import _ from "lodash";

export function useFieldErrors() {
  const fieldErrors = ref<{ [field: string]: boolean }>({});
  const formInError = computed(() => {
    return _.find(_.values(fieldErrors.value), (v) => v === true);
  });
  const setFieldInError = (field: string, inError: boolean) => {
    fieldErrors.value[field] = inError;
  };
  return { fieldErrors, formInError, setFieldInError };
}
