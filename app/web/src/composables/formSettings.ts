import { computed, ComputedRef, InjectionKey, provide, inject } from "vue";

export type FormSettings = {
  hideRequiredLabel: boolean;
  requiredLabel: string;
  requiredLabelClasses: string;
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
    computed(() => ({
      hideRequiredLabel: false,
      requiredLabel: "(required)",
      requiredLabelClasses: "",
    })),
  );
  return themeValue as unknown as FormSettings;
}
