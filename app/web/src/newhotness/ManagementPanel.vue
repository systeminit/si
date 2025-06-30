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
      v-if="incoming.length > 0"
      :edges="incoming"
      titleText="Managed By Components"
    />
    <ManagementConnectionsList
      :edges="outgoing"
      titleText="Managing Components"
      selectComponent
      :parentComponentName="component.name"
      :parentComponentId="componentId"
    />
  </div>
  <EmptyState v-else text="No management information available" icon="tools" />
</template>

<script setup lang="ts">
import { computed, PropType } from "vue";
import { useQuery } from "@tanstack/vue-query";
import { FuncRun } from "@/newhotness/api_composables/func_run";
import {
  Connection,
  EntityKind,
  BifrostComponent,
  ManagementConnections,
} from "@/workers/types/entity_kind_types";
import {
  bifrost,
  getIncomingManagement,
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
});

const mgmtFuncs = computed(
  () => props.component?.schemaVariant.mgmtFunctions ?? [],
);

const key = useMakeKey();
const args = useMakeArgs();

const componentId = computed(() => props.component?.id);

const mgmtConnectionsQuery = useQuery<ManagementConnections | null>({
  queryKey: key(EntityKind.ManagementConnections, componentId.value),
  queryFn: async () => {
    return await bifrost<ManagementConnections>(
      args(EntityKind.ManagementConnections, componentId.value),
    );
  },
});

const myIncoming = computed<Connection[]>(() => {
  if (!incomingQuery.data.value || !componentId.value) return [];

  const mine = incomingQuery.data.value.get(componentId.value);
  if (!mine) return [];
  return Object.values(mine);
});

const incomingQuery = useQuery({
  queryKey: key(EntityKind.IncomingManagementConnections),
  queryFn: async () => {
    const inc = await getIncomingManagement(
      args(EntityKind.IncomingManagementConnections),
    );
    return inc;
  },
});

const mgmtConnections = computed(() => {
  if (mgmtConnectionsQuery.data.value) {
    const { connections: outgoing } = mgmtConnectionsQuery.data.value;
    return { outgoing };
  } else {
    return {
      outgoing: [] as Connection[],
    };
  }
});

const outgoing = computed(
  () =>
    mgmtConnections.value.outgoing.map((conn) => {
      return {
        key: `mgmt-${conn.toComponentId}-${conn.fromComponentId}`,
        componentId: conn.toComponentId,
        self: "Management",
        other: "-",
      };
    }) ?? ([] as SimpleConnection[]),
);

const incoming = computed<SimpleConnection[]>(() => {
  return myIncoming.value.map((conn) => {
    return {
      key: `mgmt-${conn.toComponentId}-${conn.fromComponentId}`,
      componentId: conn.toComponentId,
      self: "Management",
      other: "-",
    };
  });
});

const managementData = computed(
  () =>
    mgmtFuncs.value.length > 0 ||
    // incoming.value.length > 0 ||
    outgoing.value.length > 0,
);
</script>
