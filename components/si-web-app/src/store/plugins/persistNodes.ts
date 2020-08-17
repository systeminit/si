import _ from "lodash";
import { Store } from "vuex";

import { RootStore } from "@/store";
import { graphqlMutation } from "@/api/apollo";

export function persistNodes(store: Store<RootStore>): void {
  store.subscribe(mutation => {
    if (mutation.type == "node/setNodePosition") {
      const id = mutation.payload.id;
      const position = mutation.payload.position;
      graphqlMutation({
        typeName: "node",
        methodName: "setPosition",
        variables: {
          id,
          position,
        },
      });
    }
  });
}
