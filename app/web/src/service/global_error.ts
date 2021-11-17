import { globalErrorMessage$ } from "@/observable/global";
import { ApiResponseError } from "@/api/sdf";

/**
 * Clear the global error message
 */
export function clear() {
  globalErrorMessage$.next(null);
}

/**
 * Set the global error message
 */
export function set(error: ApiResponseError) {
  globalErrorMessage$.next(error);
}

/**
 * Manages the global error display
 */
export const GlobalErrorService = {
  clear,
  set,
};
