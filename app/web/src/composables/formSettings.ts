import { computed, ComputedRef, InjectionKey, provide, inject } from "vue";

type FormSettings = {
  hideRequiredLabel: boolean;
};

const FORM_SETTINGS_INJECTION_KEY: InjectionKey<ComputedRef<FormSettings>> =
  Symbol();

export function setFormSettings(settings: FormSettings) {
  provide(
    FORM_SETTINGS_INJECTION_KEY,
    computed(() => settings),
  );
}

export function useFormSettings() {
  const themeValue = inject(
    FORM_SETTINGS_INJECTION_KEY,
    computed(() => ({ hideRequiredLabel: false })),
  );
  return themeValue;
}
