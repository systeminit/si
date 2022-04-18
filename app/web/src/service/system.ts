import { currentSystem } from "./system/current_system";
import { createSystem } from "./system/create_system";
import { getSystem } from "./system/get_system";
import { listSystems } from "./system/list_systems";
import { switchTo } from "./system/switch_to";
import { switchToNone } from "./system/switch_to_none";

export const SystemService = {
  currentSystem,
  createSystem,
  getSystem,
  listSystems,
  switchTo,
  switchToNone,
};
