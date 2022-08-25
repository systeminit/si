import { useObservable } from "@vueuse/rxjs";
import { getSummary } from "./qualification/get_summary";

function useQualificationSummary() {
  return useObservable(getSummary());
}

export const QualificationService = {
  useQualificationSummary,
};
