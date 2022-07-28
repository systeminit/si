import { Theme, ThemeValue } from "@/observable/theme";
import { computed, ComputedRef, InjectionKey, provide, inject } from "vue";
import { refFrom } from "vuse-rx/src";

import { ThemeService } from "@/service/theme";

const THEME_INJECTION_KEY: InjectionKey<ComputedRef<ThemeValue>> = Symbol();

export function useThemeProvider() {
  const theme = refFrom<Theme>(ThemeService.currentTheme());

  provide(
    THEME_INJECTION_KEY,
    computed(() => theme.value?.value),
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
