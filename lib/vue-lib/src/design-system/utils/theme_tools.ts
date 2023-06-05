/*
  This is set of tools for dealing with dark/light mode theming. It includes:
  - a single root theme setting based on the system preferences, or a user override persisted to localstorage
  - other components can override this theme, either with a static value, or with their own theme setting logic
  - individual components can get their current theme, which (using provide/inject) will find the theme value, whether coming from the room or a parent overriding
  - utilities which help apply sets of css classes depending on whether dark or light mode is active for a component

  All of this is needed so that we can override dark/light mode in specific regions of the page separately from the root
  which is not possible with tailwind `dark:XXX` style classes -- they only respect the root "dark" class
*/

import {
  computed,
  ComputedRef,
  InjectionKey,
  provide,
  inject,
  Ref,
  ref,
  watch,
  isRef,
} from "vue";
import storage from "local-storage-fallback";

export type ThemeValue = "dark" | "light";

const THEME_STORAGE_KEY = "SI:THEME";
const THEME_INJECTION_KEY: InjectionKey<Ref<ThemeValue>> = Symbol("THEME");

// NOTE - some issues with window here when running SSG. Tried a few things but try/catch finally worked...

// track the system theme - based off of `prefers-color-scheme`
function getSystemTheme(): ThemeValue {
  try {
    return window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light";
  } catch (err) {
    return "dark";
  }
}
export const systemTheme = ref(getSystemTheme());
try {
  window
    .matchMedia("(prefers-color-scheme: dark)")
    .addEventListener("change", () => {
      systemTheme.value = getSystemTheme();
    });
} catch (err) {}
// single user-selected theme (overriding the system theme) saved to localstorage
// we export the user-set theme directly, but we only need to use it for theme switcher components
// as most components will get current value via inject in `useTheme()`
export const userOverrideTheme = ref<ThemeValue | null>(
  (storage.getItem(THEME_STORAGE_KEY) as ThemeValue) || null,
);
// watcher to update the user theme in local-storage when it changes
watch(
  () => userOverrideTheme.value,
  (newTheme) => {
    if (newTheme) {
      storage.setItem(THEME_STORAGE_KEY, newTheme);
    } else {
      storage.removeItem(THEME_STORAGE_KEY);
    }
  },
);

// computed which returns the active theme at the root, regardless if set by user or system
const rootActiveTheme = computed(
  () => userOverrideTheme.value || systemTheme.value,
);

// makes a component provide theme value to all children, takes an arg which can be used in a few ways:
// - undefined -- provides the "root theme" value, useful for the root component or to _unset_ override from a parent
// - ThemeValue -- useful to override in a particular section
// - Ref<ThemeValue> -- useful if a component has it's own theme selection logic / picker
// could add a mode which flips the current theme...?
export function useThemeContainer(themeValue?: ThemeValue | Ref<ThemeValue>) {
  let providedTheme: Ref<ThemeValue> = rootActiveTheme;
  if (themeValue) {
    if (isRef(themeValue)) providedTheme = themeValue;
    else providedTheme = computed(() => themeValue);
  }
  provide(THEME_INJECTION_KEY, providedTheme);

  // TODO - we could try to see if this is the root component and call useHead to inject body class...
  // but useHead does not support merging for bodyAttrs yet - https://github.com/vueuse/head/issues/55
  // so we have to do this in App.vue anyway

  const themeContainerClasses = computed(
    () => `color-scheme-${providedTheme.value}`,
  ); // currently just used for scrollbars
  // we'll also return the theme being provided in case the component needs it (like to inject into head)
  return { theme: providedTheme, themeContainerClasses };
}

// used by components who just want to use the current theme
// which will usually be the root, but could be a parent that is overriding that value
export function useTheme() {
  const theme = inject(
    THEME_INJECTION_KEY,
    // dont think this default is ever actually used, but helps TS not complain
    rootActiveTheme,
  );
  return { theme };
}

// class utility to help apply specific classes based on current theme
export function themeClasses(lightClasses: string, darkClasses: string) {
  const { theme } = useTheme();
  return theme.value === "light" ? lightClasses : darkClasses;
}
