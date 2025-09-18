<template>
  <div class="grow flex flex-col">
    <!-- Mode toggle buttons -->
    <div class="flex-none p-sm border-b" :class="themeClasses('border-neutral-300', 'border-neutral-600')">
      <div class="flex flex-row gap-2">
        <NewButton
          label="Schema Visualizer"
          icon="schematics"
          tone="empty"
          :class="mode === 'schema' 
            ? themeClasses('bg-action-200', 'bg-action-900')
            : themeClasses('bg-neutral-200', 'bg-neutral-800')"
          @click="mode = 'schema'"
        />
        <NewButton
          label="Table Data"
          icon="data-table-reference"
          tone="empty"
          :class="mode === 'table' 
            ? themeClasses('bg-action-200', 'bg-action-900')
            : themeClasses('bg-neutral-200', 'bg-neutral-800')"
          @click="mode = 'table'"
        />
      </div>
    </div>

    <!-- Content area -->
    <div class="grow flex flex-col min-h-0">
      <!-- Schema Visualizer -->
      <div v-if="mode === 'schema'" class="grow">
        <GraphSchemaVisualizer
          showRelationships
          :layoutAlgorithm="'layered'"
          @nodeClick="handleNodeClick"
          @nodeDoubleClick="handleNodeDoubleClick"
          @edgeClick="handleEdgeClick"
          @selectionChange="handleSelectionChange"
          @tableViewRequest="handleTableViewRequest"
        />
      </div>

      <!-- Table Data View -->
      <div v-else-if="mode === 'table'" class="grow p-md scrollable">
        <div class="space-y-lg">
          <!-- Components Table -->
          <div class="overflow-x-auto">
            <table class="w-full border-collapse" :class="tableClasses">
              <thead>
                <tr :class="themeClasses('bg-neutral-100 text-neutral-900', 'bg-neutral-800 text-neutral-100')">
                  <th 
                    v-for="(name, index) in tableHeaders" 
                    :key="index"                     
                    :class="clsx( 
                      'p-xs border',
                      themeClasses('border-gray-300', 'border-gray-300'),
                    )"
                  >
                    {{ name }}
                  </th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="(entry, index) in tableEntries"
                  :key="index"
                  :class="clsx([
                    'cursor-pointer transition-colors',
                    themeClasses(
                      'hover:bg-neutral-50 border-b border-neutral-200',
                      'hover:bg-neutral-700 border-b border-neutral-600'
                    )
                  ])"
                >
                  <td 
                    v-for="(_, index) in tableHeaders" 
                    :key="index"
                    :class="clsx( 
                      'p-xs border',
                      themeClasses('border-gray-300', 'border-gray-300'),
                    )"
                  >
                    {{ entry[index] ?? 'undefined' }}
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, inject } from "vue";
import { useRoute, useRouter } from "vue-router";
import { Icon, themeClasses, NewButton } from "@si/vue-lib/design-system";
import { 
  ComponentInList, 
  SchemaVariant, 
  EntityKind,
  BifrostActionViewList,
} from "@/workers/types/entity_kind_types";
import type { EntityNode, RelationshipEdge, EntitySchemaKind } from "@/components/GraphSchemaVisualizer/types/schema-graph";
import GraphSchemaVisualizer from "@/components/GraphSchemaVisualizer/GraphSchemaVisualizer.vue";
import { ExploreContext } from "./types";
import clsx from "clsx";
import { useQuery } from "@tanstack/vue-query";
import { bifrost, bifrostList, useMakeArgs, useMakeKey } from "@/store/realtime/heimdall";
import { useContext } from "./logic_composables/context";
import { Listable } from "../workers/types/dbinterface";

const route = useRoute();
const router = useRouter();

// Get explore context for data
const exploreContext = inject<ExploreContext>("EXPLORE_CONTEXT");

const mode = ref<"schema" | "table">("schema"); // Default to schema view

// Sample entities for the graph visualizer
const sampleEntities = ref([
  {
    id: "schema-1",
    name: "User Management",
    entityKind: "SchemaVariant" as EntityKind,
    properties: {
      category: "Identity",
      version: "1.2.0",
      status: "healthy",
      componentCount: 3
    }
  },
  {
    id: "schema-2", 
    name: "Payment Processing",
    entityKind: "SchemaVariant" as EntityKind,
    properties: {
      category: "Finance",
      version: "2.1.0",
      status: "healthy",
      componentCount: 2
    }
  },
  {
    id: "comp-1",
    name: "Authentication Service",
    entityKind: "Component" as EntityKind,
    properties: {
      schemaVariantId: "schema-1",
      status: "success",
      hasResource: true,
      resourceType: "Service"
    }
  },
  {
    id: "comp-2",
    name: "User Database",
    entityKind: "Component" as EntityKind,
    properties: {
      schemaVariantId: "schema-1",
      status: "success", 
      hasResource: true,
      resourceType: "Database"
    }
  },
  {
    id: "comp-3",
    name: "User API Gateway",
    entityKind: "Component" as EntityKind,
    properties: {
      schemaVariantId: "schema-1",
      status: "warning",
      hasResource: true,
      resourceType: "Gateway"
    }
  },
  {
    id: "comp-4",
    name: "Payment Gateway",
    entityKind: "Component" as EntityKind,
    properties: {
      schemaVariantId: "schema-2",
      status: "success",
      hasResource: true,
      resourceType: "Gateway"
    }
  },
  {
    id: "comp-5",
    name: "Payment Processor",
    entityKind: "Component" as EntityKind,
    properties: {
      schemaVariantId: "schema-2",
      status: "success",
      hasResource: true,
      resourceType: "Service"
    }
  }
]);

// Table styling classes
const tableClasses = computed(() => [
  'border',
  themeClasses('border-neutral-300', 'border-neutral-600')
]);


const tableRowClasses = computed(() => [
  'cursor-pointer transition-colors',
  themeClasses(
    'hover:bg-neutral-50 border-b border-neutral-200',
    'hover:bg-neutral-700 border-b border-neutral-600'
  )
]);

const getStatusBadgeClasses = (status?: string) => {
  const baseClasses = ['px-2 py-1 text-xs rounded font-medium'];
  
  switch (status) {
    case 'success':
      return baseClasses.concat(themeClasses('bg-green-100 text-green-800', 'bg-green-900 text-green-200'));
    case 'failure':
      return baseClasses.concat(themeClasses('bg-red-100 text-red-800', 'bg-red-900 text-red-200'));
    case 'warning':
      return baseClasses.concat(themeClasses('bg-yellow-100 text-yellow-800', 'bg-yellow-900 text-yellow-200'));
    default:
      return baseClasses.concat(themeClasses('bg-neutral-100 text-neutral-800', 'bg-neutral-800 text-neutral-200'));
  }
};

const getFunctionStatusClasses = (status?: string) => {
  const baseClasses = ['px-2 py-1 text-xs rounded font-medium'];
  
  switch (status?.toLowerCase()) {
    case 'success':
    case 'completed':
      return baseClasses.concat(themeClasses('bg-green-100 text-green-800', 'bg-green-900 text-green-200'));
    case 'error':
    case 'failed':
      return baseClasses.concat(themeClasses('bg-red-100 text-red-800', 'bg-red-900 text-red-200'));
    case 'running':
    case 'in_progress':
      return baseClasses.concat(themeClasses('bg-blue-100 text-blue-800', 'bg-blue-900 text-blue-200'));
    default:
      return baseClasses.concat(themeClasses('bg-neutral-100 text-neutral-800', 'bg-neutral-800 text-neutral-200'));
  }
};

// Navigation functions
const navigateToComponent = (componentId: string) => {
  const params = { ...route.params };
  params.componentId = componentId;
  router.push({
    name: "new-hotness-component",
    params,
  });
};

const viewFunctionDetails = (funcRunId: string) => {
  console.log("View function details:", funcRunId);
  // TODO: Navigate to function details view when available
};

// GraphSchemaVisualizer event handlers
const handleNodeClick = (node: EntityNode) => {
  console.log("Node clicked:", node.id, node.entityKind);
};

const handleNodeDoubleClick = (node: EntityNode) => {
  console.log("Node double-clicked:", node.id);
  // Double-click could switch to detail view or table mode for that entity type
  mode.value = 'table';
};

const handleEdgeClick = (edge: RelationshipEdge) => {
  console.log("Edge clicked:", edge.id);
};

const handleSelectionChange = (selectedNodes: Set<string>) => {
  console.log("Selection changed:", Array.from(selectedNodes));
};


const tableHeaders = ref(["Id", "Name", "Variant Name"])

const tableEntries = ref([
  ["1", "Rex", "EC2 Instance"],
]);

const key = useMakeKey();
const args = useMakeArgs();

const actionViewListRaw = useQuery<BifrostActionViewList | null>({
  queryKey: key(EntityKind.ActionViewList),
  queryFn: async () =>
    await bifrost<BifrostActionViewList>(args(EntityKind.ActionViewList)),
});
const actionViewList = computed(
  () => actionViewListRaw.data.value?.actions ?? [],
);

const ctx = useContext();

const componentListQueryKind = computed(() =>
  EntityKind.ComponentList
);
const componentListQueryId = computed(() =>
  ctx.workspacePk.value,
);
const componentQueryKey = key(componentListQueryKind, componentListQueryId);
const componentListQuery = useQuery<ComponentInList[]>({
  queryKey: componentQueryKey,
  queryFn: async () => {
    const arg = args<Listable>(EntityKind.ComponentList);
    const list = await bifrostList<ComponentInList[]>(arg);
    return list ?? [];
  },
});
const componentList = computed(() => {
  return componentListQuery.data.value ?? [];
});

const handleTableViewRequest = (entityKind: EntitySchemaKind) => {
  console.log("Table view requested for:", entityKind);
  // Switch to table mode when user clicks table link in graph
  mode.value = 'table';

  if(entityKind === "Action") {
    console.log(actionViewList.value);
    tableHeaders.value = ["Id", "Kind", "State", "Description", "originatingChangeSetId", "componentId", "funcId"];
    const entries: string[][] = [];

    for (const action of actionViewList.value) {
      entries.push([
        action.id,
        action.kind,
        action.state,
        action.description ?? "",
        action.originatingChangeSetId,
        action.componentId ?? "",
        action.funcRunId ?? "",
      ])
    }

    tableEntries.value = entries;
    return;
  } else if(entityKind === "Component") {
    console.log(componentList.value);
    tableHeaders.value = ["Id", "Name", "Schema Name", "Schema Id", "Schema Variant Name", "Schema Variant Id", "Category"];
    const entries: string[][] = [];

    for (const component of componentList.value) {
      entries.push([
        component.id,
        component.name,
        component.schemaName,
        component.schemaId,
        component.schemaVariantName,
        component.schemaVariantId,
        component.schemaCategory,
      ])
    }

    tableEntries.value = entries;
    return;
  }
  else if(entityKind === "SchemaVariant") {}
  else if(entityKind === "Schema") {}
  else if(entityKind === "Function") {
  }

  mode.value = 'schema';
};

/**
 * 
  id: ComponentId;
  name: string;
  color?: null | string;
  schemaName: string;
  schemaId: SchemaId;
  // Needed for "ComponentInList" usage where the "SchemaVariant" is dropped.
  schemaVariantId: SchemaVariantId;
  schemaVariantName: string;
  schemaCategory: string;
  hasResource: boolean;
  qualificationTotals: ComponentQualificationTotals;
  inputCount: number;
  diffStatus: ComponentDiffStatus;
  toDelete: boolean;
  resourceId: string | null;
  hasSocketConnections: boolean;
 * 
 */
</script>