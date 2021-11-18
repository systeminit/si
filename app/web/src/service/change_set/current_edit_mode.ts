import { editMode$ } from "@/observable/edit_mode";

export function currentEditMode(): typeof editMode$ {
  return editMode$;
}
