<template>
  <div
    v-if="component && componentId && latestFuncRuns && managementData"
    class="p-xs flex flex-col gap-xs"
  >
    <ul class="flex flex-col gap-xs">
      <ManagementFuncCard
        v-for="func in mgmtFuncs"
        :key="func.id"
        :componentId="componentId"
        :func="func"
        :funcRun="latestFuncRuns[func.id]"
      />
    </ul>
    <ManagementConnectionsList
      :edges="incoming"
      titleText="Managed By Components"
    />
    <ManagementConnectionsList
      :edges="outgoing"
      titleText="Managing Components"
      selectComponent
      :parentComponentName="component.name"
    />
  </div>
  <EmptyState v-else text="No management information available" icon="tools" />
</template>

<script setup lang="ts">
import { computed, PropType } from "vue";
import { useQuery } from "@tanstack/vue-query";
import { FuncRun } from "@/newhotness/api_composables/func_run";
import {
  IncomingConnections,
  Connection,
  EntityKind,
  BifrostComponent,
} from "@/workers/types/entity_kind_types";
import {
  bifrost,
  getOutgoingConnections,
  useMakeArgs,
  useMakeKey,
} from "@/store/realtime/heimdall";
import EmptyState from "./EmptyState.vue";
import ManagementFuncCard from "./ManagementFuncCard.vue";
import { SimpleConnection } from "./layout_components/ConnectionLayout.vue";
import ManagementConnectionsList from "./ManagementConnectionsList.vue";

const props = defineProps({
  component: { type: Object as PropType<BifrostComponent> },
  latestFuncRuns: { type: Object as PropType<Record<string, FuncRun>> },
  connections: { type: Object as PropType<IncomingConnections> },
});

const mgmtFuncs = computed(
  () => props.component?.schemaVariant.mgmtFunctions ?? [],
);

const key = useMakeKey();
const args = useMakeArgs();

const componentId = computed(() => props.component?.id);
const enableLookup = computed(
  () => !(props.connections && "id" in props.connections),
);

const componentConnectionsQuery = useQuery<IncomingConnections | null>({
  enabled: enableLookup,
  queryKey: key(EntityKind.IncomingConnections, componentId.value),
  queryFn: async () => {
    const componentConnections = await bifrost<IncomingConnections>(
      args(EntityKind.IncomingConnections, componentId.value),
    );
    return componentConnections;
  },
});

const outgoingQuery = useQuery<Connection[]>({
  queryKey: key(EntityKind.OutgoingConnections),
  queryFn: async () => {
    const byComponents = await getOutgoingConnections(
      args(EntityKind.OutgoingConnections),
    );
    if (!componentId.value) return [];
    const mine = byComponents.get(componentId.value);
    if (!mine) return [];
    return Object.values(mine);
  },
});

const componentConnections = computed(() => {
  if (enableLookup.value && componentConnectionsQuery.data.value) {
    const { connections: incoming } = componentConnectionsQuery.data.value;
    return { incoming };
  } else if (props.connections) {
    const { connections: incoming } = props.connections;
    return { incoming };
  } else {
    return {
      incoming: [] as Connection[],
    };
  }
});

const incoming = computed(
  () =>
    componentConnections.value.incoming
      .filter((conn) => conn.kind === "management") // && conn.fromComponentId !== componentId.value
      .map((conn) => {
        return {
          key: `mgmt-${conn.toComponentId}-${conn.fromComponentId}`,
          componentId: conn.fromComponentId,
          self: "Management",
          other: "-",
        };
      }) ?? ([] as SimpleConnection[]),
);

const outgoing = computed<SimpleConnection[]>(() => {
  const outgoing = outgoingQuery.data?.value ?? [];
  return outgoing
    .filter((conn) => conn.kind === "management")
    .map((conn) => {
      return {
        key: `mgmt-${conn.toComponentId}-${conn.fromComponentId}`,
        componentId: conn.fromComponentId,
        self: "Management",
        other: "-",
      };
    });
});

const managementData = computed(
  () =>
    mgmtFuncs.value.length > 0 ||
    incoming.value.length > 0 ||
    outgoing.value.length > 0,
);
</script>
