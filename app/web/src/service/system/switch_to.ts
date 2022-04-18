import { System } from "../../api/sdf/dal/system";
import { system$ } from "../../observable/system";

export function switchTo(system: System): void {
  system$.next(system);
}
