import { reactive } from "vue";
import { ChangeSetId } from "@/api/sdf/dal/change_set";

const status = reactive<Record<ChangeSetId, "syncing" | "synced">>({});

export function useStatus() {
  return status;
}
