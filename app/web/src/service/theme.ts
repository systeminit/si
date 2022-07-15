import { Theme, theme$ } from "@/observable/theme";

// Fetch the current theme
function currentTheme(): typeof theme$ {
  return theme$;
}

/**
 * Set the theme state to light or dark
 */
function setTo(value: Theme["value"]) {
  theme$.next({ value, source: "user" });
}

/**
 * Make the whole website follow the system's theme
 */
function resetToSystems() {
  const theme: Theme = window.matchMedia("(prefers-color-scheme: dark)").matches
    ? { value: "dark", source: "system" }
    : { value: "light", source: "system" };
  theme$.next(theme);
}

/**
 * Manages dark/light theme overriding
 */
export const ThemeService = {
  setTo,
  resetToSystems,
  currentTheme,
};
