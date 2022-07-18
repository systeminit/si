import { ReplaySubject } from "rxjs";
import { persistToSession } from "@/observable/session_state";

/**
 * The type used to describe available themes
 */
export type ThemeSource = "user" | "system";
export type ThemeValue = "dark" | "light";

export interface Theme {
  value: ThemeValue;
  source: ThemeSource;
}

/**
 * The currently selected theme
 */
export const theme$ = new ReplaySubject<Theme>(1);
persistToSession("theme", theme$);

theme$.subscribe((newTheme) => {
  if (newTheme.value === "dark") document.documentElement.classList.add("dark");
  else document.documentElement.classList.remove("dark");
});

theme$.next({
  value: window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light",
  source: "system",
});
