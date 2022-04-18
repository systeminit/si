import { system$ } from "@/observable/system";

export function currentSystem(): typeof system$ {
  return system$;
}
