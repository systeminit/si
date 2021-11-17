export * from "./change_set/list_open_change_sets";
import { listOpenChangeSets } from "./change_set/list_open_change_sets";
import { createChangeSet } from "./change_set/create_change_set";
import { applyChangeSet } from "./change_set/apply_change_set";
import { getChangeSet } from "./change_set/get_change_set";
import { startEditSession } from "./change_set/start_edit_session";
import { cancelEditSession } from "./change_set/cancel_edit_session";
import { saveEditSession } from "./change_set/save_edit_session";
import { switchToHead } from "./change_set/switch_to_head";

export const ChangeSetService = {
  listOpenChangeSets,
  createChangeSet,
  applyChangeSet,
  getChangeSet,
  startEditSession,
  cancelEditSession,
  saveEditSession,
  switchToHead,
};
