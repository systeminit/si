import _ from "lodash";
import { Store } from "vuex";

import { RootStore } from "@/store";

export function persistEdits(store: Store<RootStore>): void {
  const updateFunctions: Record<string, any> = {};
  store.subscribe((mutation, state) => {
    if (mutation.type == "editor/setEditValue") {
      let id = state.editor?.selectedNode?.id;
      if (!id) {
        return;
      }
      if (!updateFunctions[id]) {
        updateFunctions[id] = _.debounce(async (entity: any) => {
          console.log("updating graphql for", { entity });
          try {
            if (mutation.payload.map) {
              let currentEditTree = _.cloneDeep(entity);
              let currentMapValue = _.get(
                currentEditTree,
                mutation.payload.path,
              );
              _.set(
                currentEditTree,
                mutation.payload.path,
                _.filter(currentMapValue, "key"),
              );
              await store.dispatch("entity/update", {
                typeName: entity["siStorable"]["typeName"],
                data: currentEditTree,
                hypotheticalState: {
                  path: mutation.payload.path,
                  value: currentMapValue,
                },
              });
            } else {
              await store.dispatch("entity/update", {
                typeName: entity["siStorable"]["typeName"],
                data: entity,
              });
            }
          } catch (err) {
            store.commit("editor/setEditSaveError", err);
          }
          store.commit("editor/setIsSaving", false);
        }, 200);
      }
      store.commit("editor/setEditSaveError", undefined);
      store.commit("editor/setIsSaving", true);
      updateFunctions[id](state.editor.editTree[id]);
    }
  });
}
