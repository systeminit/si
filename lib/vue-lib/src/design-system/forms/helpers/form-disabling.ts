import { provide, inject, InjectionKey, Ref, computed } from "vue";

const FormParentDisabledInjectionKey: InjectionKey<Ref<boolean>> =
  Symbol("formParentDisabled");

export function useDisabledBySelfOrParent(
  isDisabledDirectlyProp: Ref<boolean>,
  isGroup = false,
) {
  // we inject the parent's disabling status
  // which may be undefined if the input is not part of any parent groups
  const isParentDisabled = inject(FormParentDisabledInjectionKey, undefined);

  const isDisabledBySelfOrParent = computed(() => {
    if (isDisabledDirectlyProp.value) return true;
    if (isParentDisabled?.value) return true;
    return false;
  });

  // if the component this is used on is a possible parent
  // then we enable providing its disabled status to any children inside of it
  if (isGroup) {
    provide(FormParentDisabledInjectionKey, isDisabledBySelfOrParent);
  }

  return isDisabledBySelfOrParent;
}
