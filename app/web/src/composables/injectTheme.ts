import { ThemeValue } from "@/observable/theme";
import { computed, ComputedRef, InjectionKey, provide, inject, Ref } from "vue";

const THEME_INJECTION_KEY: InjectionKey<ComputedRef<ThemeValue>> = Symbol("THEME");

export function useThemeProvider(themeValue: Ref<ThemeValue>) {
  provide(
    THEME_INJECTION_KEY,
    computed(() => themeValue.value),
  );
}

export function useTheme() {
  const themeValue = inject(
    THEME_INJECTION_KEY,
    // dont think this default is ever actually used, but helps TS not complain
    computed(() => "light" as ThemeValue),
  );
  return themeValue;
}
