import { globalErrorMessage$ } from "@/observable/global";
import { ApiResponse, ApiResponseError } from "@/api/sdf";
import { Observable } from "rxjs";

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
 * Set the global error message if the observable requires it
 */
export function setIfError<T>(obs: Observable<ApiResponse<T>>) {
  obs.subscribe((response) => {
    if (response.error) {
      GlobalErrorService.set(response);
    }
  });
}

/**
 * Manages the global error display
 */
export const GlobalErrorService = {
  clear,
  set,
  setIfError,
};
