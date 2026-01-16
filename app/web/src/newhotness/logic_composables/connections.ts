import { computed, inject, ref } from "vue";
import { useQuery } from "@tanstack/vue-query";
import { ComponentId } from "@/api/sdf/dal/component";
import { Connection, EntityKind, IncomingConnections } from "@/workers/types/entity_kind_types";
import { bifrost, getOutgoingConnections, useMakeArgs, useMakeKey } from "@/store/realtime/heimdall";
import { assertIsDefined, Context } from "../types";
import { SimpleConnection } from "../layout_components/ConnectionLayout.vue";

export const useConnections = () => {
  const ctx = inject<Context>("CONTEXT");
  assertIsDefined<Context>(ctx);

  const key = useMakeKey();
  const args = useMakeArgs();

  // These refs are needed to react to function input.
  const incomingConnectionsProvided = ref<boolean>(false);
  const incomingQueryComponentId = ref<ComponentId>("");

  // These computed values are based on the refs and are needed for query reactivity.
  const enableLookup = computed(() => !incomingConnectionsProvided.value);
  const id = computed(() => incomingQueryComponentId.value);

  const incomingQuery = useQuery<IncomingConnections | null>({
    enabled: () => enableLookup.value && id.value !== "",
    queryKey: key(EntityKind.IncomingConnections, id),
    queryFn: async () => await bifrost<IncomingConnections>(args(EntityKind.IncomingConnections, id.value)),
  });
  const allOutgoingQuery = useQuery({
    queryKey: key(EntityKind.OutgoingConnections),
    queryFn: async () => await getOutgoingConnections(args(EntityKind.OutgoingConnections)),
  });

  return (componentId: ComponentId, connections?: IncomingConnections) => {
    return computed(() => {
      if (connections && "id" in connections) {
        incomingConnectionsProvided.value = true;
      } else {
        incomingQueryComponentId.value = componentId;
        incomingConnectionsProvided.value = false;
      }

      // TODO(nick): decide if this needs to be an inner computed or not.
      const incomingConnections = computed(() => {
        if (!incomingConnectionsProvided.value && incomingQuery.data.value) {
          const { connections: incoming } = incomingQuery.data.value;
          return incoming;
        } else if (connections) {
          const { connections: incoming } = connections;
          return incoming;
        } else {
          return [] as Connection[];
        }
      });

      // TODO(nick): decide if this needs to be an inner computed or not.
      const outgoingConnections = computed<Connection[]>(() => {
        if (!allOutgoingQuery.data.value) return [];
        const mine = allOutgoingQuery.data.value.get(componentId);
        if (!mine) return [];
        return Object.values(mine);
      });

      const incoming: SimpleConnection[] = incomingConnections.value.map((conn) => {
        if (conn.kind === "management") {
          // FIXME(nick,jobelenus): we should split the connection type into two now that
          // management connections have their own MV.
          return {
            key: `mgmt-${conn.toComponentId}-${conn.fromComponentId}`,
            componentId: conn.fromComponentId,
            self: "Management",
            other: "-",
          };
        } else {
          return {
            key: `${conn.toAttributeValueId}-${conn.toComponentId}-${conn.fromComponentId}-${conn.fromAttributeValueId}`,
            componentId: conn.fromComponentId,
            self: conn.toAttributeValuePath,
            other: conn.fromAttributeValuePath,
          };
        }
      });

      const outgoing: SimpleConnection[] = outgoingConnections.value.map((conn) => {
        if (conn.kind === "management") {
          // FIXME(nick,jobelenus): we should split the connection type into two now that
          // management connections have their own MV.
          return {
            key: `mgmt-${conn.toComponentId}-${conn.fromComponentId}`,
            componentId: conn.fromComponentId,
            self: "Management",
            other: "-",
          };
        } else {
          return {
            key: `${conn.toAttributeValueId}-${conn.toComponentId}-${conn.fromComponentId}-${conn.fromAttributeValueId}`,
            componentId: conn.fromComponentId,
            self: conn.toAttributeValuePath,
            other: conn.fromAttributeValuePath,
          };
        }
      });

      return { incoming, outgoing };
    });
  };
};
