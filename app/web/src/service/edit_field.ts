import { getEditFields } from "./edit_field/get_edit_fields";
import { insertFromEditField } from "./edit_field/insert_from_edit_field";
import { removeFromEditField } from "./edit_field/remove_from_edit_field";
import { updateFromEditField } from "./edit_field/update_from_edit_field";

export const EditFieldService = {
  getEditFields,
  insertFromEditField,
  removeFromEditField,
  updateFromEditField,
};
