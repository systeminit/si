import { theme$ } from "@/observable/theme";

/**
 * Set the theme state to light or dark
 */
function setTo(theme: Theme) {
  theme$.next(theme);

  if (theme === "dark") document.documentElement.classList.add("dark");
  else document.documentElement.classList.remove("dark");
}

/**
 * Make the whole website follow the system's theme
 */
export function resetToSystems() {
  const theme: Theme = window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";

  setTo(theme);
  console.log(theme);
}

/**
 * Manages dark/light theme overriding
 */
export const ThemeService = {
  setTo,
  resetToSystems,
};
