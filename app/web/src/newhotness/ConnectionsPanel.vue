<template>
  <div
    v-if="incoming.length > 0 || outgoing.length > 0"
    class="border border-neutral-500 rounded bg-neutral-900 pb-sm"
  >
    <template v-if="incoming.length">
      <ConnectionLayout label="Inputs" :connections="incoming" />
    </template>
    <template v-if="outgoing.length">
      <ConnectionLayout label="Outputs" :connections="outgoing" />
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, Reactive } from "vue";
import { useQuery } from "@tanstack/vue-query";
import {
  Component,
  BifrostComponentConnections,
  BifrostConnection,
  OutgoingConnections,
} from "@/workers/types/dbinterface";
import {
  bifrost,
  getOutgoingConnections,
  useMakeArgs,
  useMakeKey,
} from "@/store/realtime/heimdall";
import ConnectionLayout, {
  SimpleConnection,
} from "./layout_components/ConnectionLayout.vue";

const props = defineProps<{
  component: Component;
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
  queryKey: key("IncomingConnections", componentId),
  queryFn: async () => {
    const componentConnections = await bifrost<BifrostComponentConnections>(
      args("IncomingConnections", componentId.value),
    );
    return componentConnections;
  },
});

const outgoingQuery = useQuery<Reactive<OutgoingConnections>>({
  queryKey: key("OutgoingConnections"),
  queryFn: async () => {
    return await getOutgoingConnections(args("OutgoingConnections"));
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
      return {
        key: `${conn.toAttributeValueId}-${conn.toComponent.id}-${conn.fromComponent.id}-${conn.fromAttributeValueId}`,
        component: conn.fromComponent,
        componentId: conn.fromComponent.id,
        self: conn.toAttributeValuePath,
        other: conn.fromAttributeValuePath,
      };
    }) ?? ([] as SimpleConnection[]),
);

const outgoing = computed<SimpleConnection[]>(() => {
  const outgoing = outgoingQuery.data?.value?.get(componentId.value) ?? [];
  return outgoing.map((conn) => {
    return {
      key: `${conn.toAttributeValueId}-${conn.toComponent.id}-${conn.fromComponent.id}-${conn.fromAttributeValueId}`,
      component: conn.fromComponent,
      componentId: conn.fromComponent.id,
      self: conn.toAttributeValuePath,
      other: conn.fromAttributeValuePath,
    };
  });
});
</script>
