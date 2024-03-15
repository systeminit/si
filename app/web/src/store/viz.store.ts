import { defineStore } from "pinia";
import { ApiRequest, addStoreHooks } from "@si/vue-lib/pinia";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { Visibility } from "@/api/sdf/dal/visibility";
import { nilId } from "@/utils/nilId";

export type NodeKind = "Category" | "Content" | "Func" | "Ordering" | "Prop";

export type ContentKind =
  | "Root"
  | "ActionPrototype"
  | "AttributePrototype"
  | "AttributePrototypeArgument"
  | "AttributeValue"
  | "Component"
  | "FuncArg"
  | "Func"
  | "InputSocket"
  | "OutputSocket"
  | "Prop"
  | "Schema"
  | "SchemaVariant"
  | "StaticArgumentValue"
  | "ValidationPrototype";

export type EdgeWeightKind =
  | "Use"
  | "Action"
  | "ActionPrototype"
  | "AuthenticationPrototype"
  | "Contain"
  | "FrameContains"
  | "Ordering"
  | "Ordinal"
  | "Prop"
  | "Prototype"
  | "PrototypeArgument"
  | "PrototypeArgumentValue"
  | "Proxy"
  | "Root"
  | "Socket"
  | "SocketValue"
  | "Use";

export interface VizResponse {
  edges: {
    from: string;
    to: string;
    edgeWeightKind: EdgeWeightKind | null;
  }[];

  nodes: {
    id: string;
    nodeKind: NodeKind;
    contentKind: ContentKind | null;
    name: string | null;
  }[];

  rootNodeId: string;
}

export const useVizStore = () => {
  const changeSetStore = useChangeSetsStore();
  const selectedChangeSetId = changeSetStore.selectedChangeSetId;
  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspacePk;

  // TODO(nick): we need to allow for empty visibility here. Temporarily send down "nil" to mean that we want the
  // query to find the default change set.
  const visibility: Visibility = {
    visibility_change_set_pk:
      selectedChangeSetId ?? changeSetStore.headChangeSetId ?? nilId(),
  };

  return addStoreHooks(
    defineStore(
      `ws${workspaceId || "NONE"}/cs${selectedChangeSetId || "NONE"}/viz`,
      {
        state: () => ({
          edges: [],
          nodes: [],
        }),
        getters: {
          nodes: (state) => state.nodes,
          edges: (state) => state.edges,
        },
        actions: {
          async FETCH_VIZ() {
            return new ApiRequest<VizResponse>({
              url: "/graphviz/nodes_edges",
              params: { ...visibility },
            });
          },
          async FETCH_SCHEMA_VARIANT_VIZ(schemaVariantId: string) {
            return new ApiRequest<VizResponse>({
              url: "/graphviz/schema_variant",
              params: { schemaVariantId, ...visibility },
            });
          },
        },
      },
    ),
  )();
};
