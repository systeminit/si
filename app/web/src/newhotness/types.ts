import { Ref } from "vue";

export interface WSCS {
  workspacePk: Ref<string, string>;
  changeSetId: Ref<string, string>;
}
