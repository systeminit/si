import { ReplaySubject } from "rxjs";
import { persistToSession } from "@/observable/session_state";

/**
 * The type used to describe available themes
 */
export type ThemeSource = "user" | "system";
export type ThemeValue = "dark" | "light";

interface Theme {
  value: ThemeValue;
  source: ThemeSource;
}

/**
 * The currently selected theme
 */
export const theme$ = new ReplaySubject<Theme>(1);

theme$.next({
  value: window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light",
  source: "system",
});
persistToSession("theme", theme$);
