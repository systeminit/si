/* eslint-disable @typescript-eslint/no-explicit-any,no-control-regex */
/**
 * Some helpers to handle validations in custom input components and checking the validation status on groups of those components.
 *
 * For individual components, this helps us show error messages only once they have been "touched", meaning the user focused and then blurred
 * focus on the input OR they clicked submit on the form.
 *
 * For a group (like a form or a form section) this will automatically register all child components found within (via provide/inject)
 * and then expose computed props about whether any of the inputs are invalid, and methods to "touch" all inputs at once, which can be called
 * on form submit.
 *
 * This pattern was sort of inspired by vuelidate (https://vuelidate.js.org/) however this is more about having inputs that know some
 * of their own validation rules and display their own error messages, and then auto-registering with their parents, rather than having the
 * parent own all of the validation rules.
 */

import {
  computed,
  ref,
  provide,
  inject,
  InjectionKey,
  onMounted,
  onBeforeUnmount,
  getCurrentInstance,
  ComponentInternalInstance,
  Ref,
  reactive,
} from "vue";
import * as _ from "lodash-es";
import { toMs } from "ms-typescript";

type ValidatedInputGroupRegistrar = {
  register(component: ComponentInternalInstance, isGroup: boolean): void;
  unregister(component: ComponentInternalInstance, isGroup: boolean): void;
};
export type InputValidationRule = {
  fn: (val: any) => boolean;
  message: string | ((val: any) => string);
};
const ValidationGroupInjectionKey: InjectionKey<ValidatedInputGroupRegistrar> =
  Symbol("ValidatedInputGroupRegistrar");

/**
 * handles injecting the parent group registrar into a component
 * and then registering/unregistering the child with that parent on mount/unmount
 * this is used by both individual inputs and groups themselves (to enable nesting of groups)
 */
function useValidatedInputGroupItem(isGroup = false) {
  const thisComponent = getCurrentInstance();
  const inputGroupRegistrar = inject(ValidationGroupInjectionKey, undefined);
  onMounted(() => {
    if (inputGroupRegistrar && thisComponent) {
      inputGroupRegistrar.register(thisComponent, isGroup);
    }
  });
  onBeforeUnmount(() => {
    if (inputGroupRegistrar && thisComponent) {
      inputGroupRegistrar.unregister(thisComponent, isGroup);
    }
  });
}

/**
 * composition function to handle getting validation status for a group of inputs
 * automatically registers all children found within the component
 * */
export function useValidatedInputGroup() {
  const children = reactive({
    groups: [] as ComponentInternalInstance[],
    inputs: [] as ComponentInternalInstance[],
  });
  const allChildren = computed(() => [...children.groups, ...children.inputs]);

  const allTouched = computed(
    () =>
      _.every(
        children.inputs,
        (c) => c?.exposed?.validationState?.isTouched === true,
      ) &&
      _.every(
        children.groups,
        (g) => g?.exposed?.validationState?.allTouched === true,
      ),
  );

  const isInvalid = computed(
    () =>
      _.some(
        children.inputs,
        (c) => c?.exposed?.validationState?.isTouched === true,
      ) ||
      _.some(
        children.groups,
        (g) => g?.exposed?.validationState?.allTouched === true,
      ),
  );

  const isError = computed(
    () =>
      _.some(
        children.inputs,
        (c) => c?.exposed?.validationState?.isError === true,
      ) ||
      _.some(
        children.groups,
        (g) => g?.exposed?.validationState?.isError === true,
      ),
  );

  // TODO: add more logic for handling warnings vs errors

  const validationMethods = {
    touchAll() {
      _.each(children.inputs, (c) => c?.exposed?.validationMethods.touch());
      _.each(children.groups, (g) => g?.exposed?.validationMethods.touchAll());
    },
    resetAll() {
      _.each(children.inputs, (c) => c?.exposed?.validationMethods.reset());
      _.each(children.groups, (g) => g?.exposed?.validationMethods.resetAll());
    },
    hasError() {
      // touches all fields and then return if the group has any errors
      // useful when user clicks a "submit" button so any errors will be shown and submit button disabled until they are fixed
      validationMethods.touchAll();
      return isError.value;
    },
  };

  provide<ValidatedInputGroupRegistrar>(ValidationGroupInjectionKey, {
    register(component, isGroup) {
      if (isGroup) children.groups.push(component as any);
      else children.inputs.push(component as any);
    },
    unregister(component, isGroup) {
      // not sure if uid is the best method here, but it works
      if (isGroup)
        children.groups = _.reject(children.groups, { uid: component.uid });
      else children.inputs = _.reject(children.inputs, { uid: component.uid });
    },
  });

  // groups can be nested, so groups must register with their parents as well
  useValidatedInputGroupItem(true);

  // create a new reactive object holding the computed statuses, just for convenience within components
  const validationState = reactive({
    allTouched,
    isError,
    isInvalid,
  });

  // unfortunately, calling defineExpose from a composition funciton doesn't work :(
  // so we have to expose them from the component itself
  // defineExpose({
  //   validationState,
  //   validationMethods,
  // });

  return {
    validationChildren: allChildren,
    validationState,
    validationMethods,
  };
}

/**
 * composition function to use within custom inputs so they will know how validate themselves
 * and be aware of if they have been "touched" or not
 */
export function useValidatedInput(
  currentValue: Ref<any>,
  validationRules: Ref<InputValidationRule[]>,
) {
  const isTouched = ref(false);
  const validationErrors = computed(() => {
    const runValidations = _.mapValues(validationRules.value, (rule: any) => {
      if (rule.fn(currentValue.value) === false) return rule.message;
      return null;
    });
    // pickby here removes empty keys
    return _.pickBy(runValidations);
  });
  const isInvalid = computed(() => _.keys(validationErrors.value).length > 0);
  const isError = computed(() => isTouched.value && isInvalid.value);
  // takes the first error... we could decide to expose them all?
  const errorMessage = computed(() => _.values(validationErrors.value)[0]);

  useValidatedInputGroupItem(); // registers the input with any parent groups

  const validationState = reactive({
    isTouched,
    isInvalid,
    isError,
    errorMessage,
  });

  // add some methods to interact with touched so we dont need to expose it directly
  const validationMethods = {
    touch() {
      isTouched.value = true;
    },
    reset() {
      isTouched.value = false;
    },
  };

  // again we must expose validationState and validationMethods
  // but we can't do it here, since it must be called from the component :(
  // defineExpose({ validationState, validationMethods });

  return { validationState, validationMethods };
}

// VALIDATORS ----------------------------------------------------------------------------
// just providing some common helpers that will be used often
// previously was relying on vuelidate's validators but was running into some type issues
// most of this is copied from vuelidate but simplified a bit
// see https://github.com/vuelidate/vuelidate/tree/master/src/validators
// TODO: probably find an existing small library to help with this, although this is very straightforward
// or move into another file

function req(value: any) {
  if (Array.isArray(value)) return !!value.length;
  if (value === undefined || value === null) {
    return false;
  }

  if (value === false) {
    return true;
  }

  if (value instanceof Date) {
    // invalid date won't pass
    return !Number.isNaN(value.getTime());
  }

  if (typeof value === "object") {
    // eslint-disable-next-line no-restricted-syntax, guard-for-in, no-unreachable-loop
    for (const v in value) return true;
    return false;
  }

  return !!String(value).length;
}
const URL_REGEX =
  /^(?:(?:https?|ftp):\/\/)(?:\S+(?::\S*)?@)?(?:(?!(?:10|127)(?:\.\d{1,3}){3})(?!(?:169\.254|192\.168)(?:\.\d{1,3}){2})(?!172\.(?:1[6-9]|2\d|3[0-1])(?:\.\d{1,3}){2})(?:[1-9]\d?|1\d\d|2[01]\d|22[0-3])(?:\.(?:1?\d{1,2}|2[0-4]\d|25[0-5])){2}(?:\.(?:[1-9]\d?|1\d\d|2[0-4]\d|25[0-4]))|(?:(?:[a-z\u00a1-\uffff0-9]-*)*[a-z\u00a1-\uffff0-9]+)(?:\.(?:[a-z\u00a1-\uffff0-9]-*)*[a-z\u00a1-\uffff0-9]+)*(?:\.(?:[a-z\u00a1-\uffff]{2,})))(?::\d{2,5})?(?:[/?#]\S*)?$/i;
const EMAIL_REGEX =
  /^(?:[A-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[A-z0-9!#$%&'*+/=?^_`{|}~-]+)*|"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|[\x01-\x09\x0b\x0c\x0e-\x7f])*")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9]{2,}(?:[a-z0-9-]*[a-z0-9])?|\[(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?|[a-z0-9-]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\])$/i;

export const validators = {
  required: (value: any) =>
    req(typeof value === "string" ? value.trim() : value),
  url: (value: any) => !req(value) || URL_REGEX.test(value),
  email: (value: any) => !req(value) || EMAIL_REGEX.test(value),
  timeString: (value: any) =>
    value &&
    value.length > 0 &&
    Number.isNaN(Number(value)) &&
    !Number.isNaN(toMs(value)) &&
    toMs(value) !== 0,
  equals: (mustEqual: any) => (value: any) => value === mustEqual,
  regex: (regexRaw: RegExp | string) => {
    const regex = _.isString(regexRaw) ? new RegExp(regexRaw) : regexRaw;
    return (value: any) => !req(value) || regex.test(value);
  },
  minLength: (minLength: number) => {
    return (value: any) =>
      !req(typeof value === "string" ? value.trim() : value) ||
      value.length >= minLength;
  },
  maxLength: (maxLength: number) => {
    return (value: any) =>
      !req(typeof value === "string" ? value.trim() : value) ||
      value.length <= maxLength;
  },
};
