import { ref } from "vue";

const status = ref<"syncing" | "synced" | undefined>(undefined);

export function useStatus() {
  return status;
}
