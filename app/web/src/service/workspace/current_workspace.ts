import { workspace$ } from "@/observable/workspace";

export function currentWorkspace(): typeof workspace$ {
  return workspace$;
}
