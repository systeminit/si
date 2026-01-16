import { ref, Ref, computed } from "vue";
import * as _ from "lodash-es";

export const VALID_USERNAME_REGEX = /^[a-z0-9._-]+$/i;

export type ValidatorArray = Array<{
  id: string;
  check: (v: string) => boolean;
  message: string;
}>;
export type ErrorsArray = Array<{ id: string; message: string }>;

export const useValidations = (
  alwaysValidate: Ref<boolean>,
  onSetDirty: () => void,
  onError: (inError: boolean) => void,
) => {
  const dirty = ref(false);
  const inError = ref(false);

  const setDirty = () => {
    dirty.value = true;
    onSetDirty();
  };

  const reallyDirty = computed(() => (alwaysValidate.value ? true : dirty.value));

  const setInError = (errors: ErrorsArray) => {
    let nextInError = false;
    if (errors.length === 1) {
      if (_.find(errors, (e) => e.id === "required")) {
        if (dirty.value) {
          nextInError = true;
        }
      } else {
        nextInError = true;
      }
    } else if (errors.length > 1) {
      nextInError = true;
    }
    inError.value = nextInError;
    onError(inError.value);
  };

  return {
    inError,
    reallyDirty,
    setDirty,
    setInError,
  };
};
