import { ReplaySubject } from "rxjs";
import { ApiResponseError } from "@/api/sdf";

/**
 * The currently displayed global error message
 */
export const globalErrorMessage$ = new ReplaySubject<ApiResponseError | null>(
  1,
);
globalErrorMessage$.next(null);
