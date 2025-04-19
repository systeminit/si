import { Ref, ref } from "vue";

export interface Toggle {
  open: Ref<boolean, boolean>;
  toggle: (e: Event) => void;
}

/**
 * Encapsulates the ref and fn to keep components cleaner
 * @returns { Toggle }
 */
export const useToggle = (): Toggle => {
  const open = ref(true);

  const toggle = (_e: Event) => {
    open.value = !open.value;
  };

  return { open, toggle };
};
