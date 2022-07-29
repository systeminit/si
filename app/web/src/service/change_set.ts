import { withLatestFrom } from "rxjs";
import { listOpenChangeSets } from "./change_set/list_open_change_sets";
import { createChangeSet } from "./change_set/create_change_set";
import { applyChangeSet } from "./change_set/apply_change_set";
import { getChangeSet } from "./change_set/get_change_set";
import { startEditSession } from "./change_set/start_edit_session";
import { cancelEditSession } from "./change_set/cancel_edit_session";
import { cancelAndStartEditSession } from "./change_set/cancel_and_start_edit_session";
import { saveEditSession } from "./change_set/save_edit_session";
import { saveEditSessionAndApplyChangeSet } from "./change_set/save_edit_session_and_apply_change_set";
import { saveAndStartEditSession } from "./change_set/save_and_start_edit_session";
import { switchToHead } from "./change_set/switch_to_head";
import { currentChangeSet } from "./change_set/current_change_set";
import { currentEditSession } from "./change_set/current_edit_session";
import { currentEditMode } from "./change_set/current_edit_mode";
import { updateSelectedChangeSet } from "./change_set/update_selected_change_set";
import {
  changeSet$,
  eventChangeSetApplied$,
  eventChangeSetCanceled$,
} from "@/observable/change_set";
import { GlobalErrorService } from "@/service/global_error";
import { user$ } from "@/observable/user";
import _ from "lodash";

export const ChangeSetService = {
  currentEditMode,
  currentChangeSet,
  currentEditSession,
  listOpenChangeSets,
  createChangeSet,
  applyChangeSet,
  getChangeSet,
  startEditSession,
  cancelEditSession,
  cancelAndStartEditSession,
  saveEditSession,
  saveEditSessionAndApplyChangeSet,
  saveAndStartEditSession,
  switchToHead,
  updateSelectedChangeSet,
};

/**
 * When a the current change set is applied, we need to show an error if we
 * magically change the state for the user.
 */
eventChangeSetApplied$
  .pipe(withLatestFrom(user$, changeSet$))
  .subscribe(([event, user, changeSet]) => {
    if (event && user && changeSet) {
      if (event.payload.data == changeSet.pk) {
        if (event.history_actor == "SystemInit") {
          GlobalErrorService.set({
            error: {
              message: "Active change set was applied by System Initiative",
              code: 42,
              statusCode: 42,
            },
          });
        } else if (!_.isEqual(event.history_actor, { User: user.id })) {
          GlobalErrorService.set({
            error: {
              message: "Active change set was applied by another user",
              code: 42,
              statusCode: 42,
            },
          });
        }
        ChangeSetService.switchToHead();
      }
    }
  });

eventChangeSetCanceled$
  .pipe(withLatestFrom(user$, changeSet$))
  .subscribe(([event, user, changeSet]) => {
    if (event && user && changeSet) {
      if (event.payload.data == changeSet.pk) {
        if (event.history_actor == "SystemInit") {
          GlobalErrorService.set({
            error: {
              message: "Active change set was canceled by System Initiative",
              code: 42,
              statusCode: 42,
            },
          });
        } else if (!_.isEqual(event.history_actor, { User: user.id })) {
          GlobalErrorService.set({
            error: {
              message: "Active change set was canceled by another user",
              code: 42,
              statusCode: 42,
            },
          });
        }
        ChangeSetService.switchToHead();
      }
    }
  });
