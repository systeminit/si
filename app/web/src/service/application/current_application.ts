import { application$ } from "@/observable/application";

export function currentApplication(): typeof application$ {
  return application$;
}
