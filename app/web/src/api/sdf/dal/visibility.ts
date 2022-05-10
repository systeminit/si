export const NO_CHANGE_SET_PK = -1;
export const NO_EDIT_SESSION_PK = -1;

export interface Visibility {
  visibility_change_set_pk: number;
  visibility_edit_session_pk: number;
  visibility_deleted_at?: Date;
}
