import { defineStore } from "pinia";
import { ApiRequest, addStoreHooks } from "@si/vue-lib/pinia";
import { reactive } from "vue";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { Visibility } from "@/api/sdf/dal/visibility";
import { nilId } from "@/utils/nilId";

export type NodeKind = "Category" | "Content" | "Func" | "Ordering" | "Prop" | "Component";

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

export interface VizResponse {
  edges: {
    from: string;
    to: string;
    edgeWeightKind: string;
  }[];

  nodes: {
    id: string;
    nodeKind: NodeKind;
    contentKind: ContentKind | null;
    name: string | null;
  }[];

  rootNodeId: string;
}

export const useVizStore = addStoreHooks(
  undefined,
  undefined,
  // TODO look into whether this ID on the store needs to be more dynamic
  defineStore(`ws/viz`, () => {
    const changeSetStore = useChangeSetsStore();
    const selectedChangeSetId = changeSetStore.selectedChangeSetId;

    // TODO(nick): we need to allow for empty visibility here. Temporarily send down "nil" to mean that we want the
    // query to find the default change set.
    const visibility: Visibility = {
      visibility_change_set_pk: selectedChangeSetId ?? changeSetStore.headChangeSetId ?? nilId(),
    };
    const data: VizResponse = reactive({
      edges: [],
      nodes: [],
      rootNodeId: "",
    });

    function onSuccess(response: VizResponse): void {
      data.nodes.splice(0, data.nodes.length);
      data.edges.splice(0, data.edges.length);
      data.edges.push(...response.edges);
      data.nodes.push(...response.nodes);
      data.rootNodeId = response.rootNodeId;
    }

    async function LOAD_COMPONENTS() {
      return new ApiRequest<VizResponse>({
        url: "/graphviz/components",
        params: { ...visibility },
        onSuccess,
      });
    }

    async function LOAD_VARIANTS(schemaVariantId: string | null) {
      return schemaVariantId === null
        ? new ApiRequest<VizResponse>({
            url: "/graphviz/nodes_edges",
            params: { ...visibility },
            onSuccess,
          })
        : new ApiRequest<VizResponse>({
            url: "/graphviz/schema_variant",
            params: { schemaVariantId, ...visibility },
            onSuccess,
          });
    }

    async function DEBUG_NODE(nodeId: string) {
      return new ApiRequest({
        url: "/node_debug",
        params: { id: nodeId, ...visibility },
      });
    }

    return {
      edges: data.edges,
      nodes: data.nodes,
      LOAD_VARIANTS,
      LOAD_COMPONENTS,
      DEBUG_NODE,
    };
  }),
);
