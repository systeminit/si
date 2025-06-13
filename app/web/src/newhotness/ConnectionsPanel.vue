<template>
  <div
    v-if="incoming.length > 0 || outgoing.length > 0"
    :class="
      clsx(
        'border rounded pb-sm',
        themeClasses(
          'border-neutral-400 bg-white',
          'border-neutral-600 bg-neutral-900',
        ),
      )
    "
  >
    <template v-if="incoming.length">
      <ConnectionLayout label="Inputs" :connections="incoming" />
    </template>
    <template v-if="outgoing.length">
      <ConnectionLayout label="Outputs" :connections="outgoing" />
    </template>
  </div>
  <!-- TODO(Wendy) - separate empty states for input and output connections? -->
  <EmptyState v-else icon="input-connection" text="No connections yet" />
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useQuery } from "@tanstack/vue-query";
import clsx from "clsx";
import { themeClasses } from "@si/vue-lib/design-system";
import {
  BifrostComponentConnections,
  BifrostComponentInList,
  BifrostConnection,
  EntityKind,
} from "@/workers/types/entity_kind_types";
import {
  bifrost,
  getOutgoingConnections,
  useMakeArgs,
  useMakeKey,
} from "@/store/realtime/heimdall";
import ConnectionLayout, {
  SimpleConnection,
} from "./layout_components/ConnectionLayout.vue";
import EmptyState from "./EmptyState.vue";

const props = defineProps<{
  component: BifrostComponentInList;
  connections?: BifrostComponentConnections;
}>();

const key = useMakeKey();
const args = useMakeArgs();

const componentId = computed(() => props.component.id);
const enableLookup = computed(
  () => !(props.connections && "id" in props.connections),
);

const componentConnectionsQuery = useQuery<BifrostComponentConnections | null>({
  enabled: enableLookup,
  queryKey: key(EntityKind.IncomingConnections, componentId),
  queryFn: async () => {
    const componentConnections = await bifrost<BifrostComponentConnections>(
      args(EntityKind.IncomingConnections, componentId.value),
    );
    return componentConnections;
  },
});

const outgoingQuery = useQuery<BifrostConnection[]>({
  queryKey: key(EntityKind.OutgoingConnections),
  queryFn: async () => {
    const byComponents = await getOutgoingConnections(
      args(EntityKind.OutgoingConnections),
    );
    const mine = byComponents.get(props.component.id);
    if (!mine) return [];
    return Object.values(mine);
  },
});

const componentConnections = computed(() => {
  if (enableLookup.value && componentConnectionsQuery.data.value) {
    const { incoming } = componentConnectionsQuery.data.value;
    return { incoming };
  } else if (props.connections) {
    const { incoming } = props.connections;
    return { incoming };
  } else {
    return {
      incoming: [] as BifrostConnection[],
    };
  }
});

// these two shouldn't be the same shape?
const incoming = computed(
  () =>
    componentConnections.value.incoming.map((conn) => {
      if (conn.kind === "management") {
        return {
          key: `mgmt-${conn.toComponent.id}-${conn.fromComponent.id}`,
          component: conn.fromComponent,
          componentId: conn.fromComponent.id,
          self: "Management",
          other: "-",
        };
      } else {
        return {
          key: `${conn.toAttributeValueId}-${conn.toComponent.id}-${conn.fromComponent.id}-${conn.fromAttributeValueId}`,
          component: conn.fromComponent,
          componentId: conn.fromComponent.id,
          self: conn.toAttributeValuePath,
          other: conn.fromAttributeValuePath,
        };
      }
    }) ?? ([] as SimpleConnection[]),
);

const outgoing = computed<SimpleConnection[]>(() => {
  const outgoing = outgoingQuery.data?.value ?? [];
  return outgoing.map((conn) => {
    if (conn.kind === "management") {
      return {
        key: `mgmt-${conn.toComponent.id}-${conn.fromComponent.id}`,
        component: conn.fromComponent,
        componentId: conn.fromComponent.id,
        self: "Management",
        other: "-",
      };
    } else {
      return {
        key: `${conn.toAttributeValueId}-${conn.toComponent.id}-${conn.fromComponent.id}-${conn.fromAttributeValueId}`,
        component: conn.fromComponent,
        componentId: conn.fromComponent.id,
        self: conn.toAttributeValuePath,
        other: conn.fromAttributeValuePath,
      };
    }
  });
});
</script>
