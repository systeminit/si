import _ from "lodash";
import { Store } from "vuex";

import { RootStore } from "@/store";

export function persistEdits(store: Store<RootStore>): void {
  const updateFunctions: Record<string, any> = {};
  store.subscribe((mutation, state) => {
    if (mutation.type == "node/setFieldValue") {
      let currentNode = state.node.current;
      if (!currentNode) {
        return;
      }
      let id = currentNode.id;
      if (id && !updateFunctions[id]) {
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
              });
            } else {
              await store.dispatch("entity/update", {
                typeName: entity["siStorable"]["typeName"],
                data: entity,
              });
            }
            if (store.state.changeSet.current?.id) {
              await store.dispatch("changeSet/get", {
                changeSetId: store.state.changeSet.current?.id,
              });
            }
          } catch (err) {
            store.commit("editor/setEditSaveError", err);
          }
          store.commit("editor/setIsSaving", false);
        }, 500);
      }
      store.commit("editor/setEditSaveError", undefined);
      store.commit("editor/setIsSaving", true);
      let entity;
      if (state.changeSet.current?.id) {
        let currentChangeSetId = state.changeSet.current.id;
        if (currentNode.display[currentChangeSetId]) {
          entity = currentNode.display[currentChangeSetId];
        } else {
          entity = currentNode.display["saved"];
        }
      } else {
        entity = currentNode.display["saved"];
      }

      // @ts-ignore
      updateFunctions[id](entity);
    }
  });
}
