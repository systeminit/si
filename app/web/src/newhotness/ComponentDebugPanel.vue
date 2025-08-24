<template>
  <div
    data-testid="component-debug-panel"
    class="flex flex-col h-full text-sm"
    :class="themeClasses('text-neutral-700', 'text-neutral-300')"
  >
    <div class="sticky top-0 z-10 p-xs" :class="bgClass">
      <SiSearch
        ref="searchRef"
        v-model="searchQuery"
        placeholder="Search debug fields"
        :tabIndex="0"
        :borderBottom="false"
        variant="new"
      />
    </div>

    <div class="flex-1 overflow-y-auto p-xs space-y-xs">
      <div
        v-if="componentDebugQuery.isPending.value"
        class="text-center py-xs opacity-60"
      >
        Loading debug data...
      </div>

      <div
        v-else-if="componentDebugQuery.isError.value"
        class="text-center py-xs"
        :class="destructiveClass"
      >
        Error: {{ componentDebugQuery.error.value?.message }}
      </div>

      <div v-else-if="componentData" class="space-y-xs">
        <!-- Component Details -->
        <div class="border rounded p-xs" :class="cardClass">
          <h3 class="text-base font-semibold mb-xs" :class="headerClass">
            Component Details
          </h3>
          <div class="space-y-2xs text-xs">
            <div>
              <span class="font-semibold">Name:</span> {{ componentData.name }}
            </div>
            <div>
              <span class="font-semibold">Schema ID:</span>
              <span class="font-mono text-2xs break-all">{{
                componentData.schemaVariantId
              }}</span>
            </div>
            <div v-if="componentData.parentId">
              <span class="font-semibold">Parent ID:</span>
              <span class="font-mono text-2xs break-all">{{
                componentData.parentId
              }}</span>
            </div>
          </div>
        </div>

        <!-- Attributes Table -->
        <div class="border rounded p-xs" :class="cardClass">
          <h3 class="text-base font-semibold mb-xs" :class="headerClass">
            Attributes ({{ filteredAttributes.length }})
          </h3>
          <div v-if="filteredAttributes.length" class="overflow-x-auto">
            <table class="min-w-full text-xs">
              <thead>
                <tr
                  class="border-b"
                  :class="
                    themeClasses('border-neutral-200', 'border-neutral-700')
                  "
                >
                  <th
                    class="text-left p-2xs font-semibold whitespace-nowrap w-8"
                  ></th>
                  <th class="text-left p-2xs font-semibold whitespace-nowrap">
                    Path
                  </th>
                  <th class="text-left p-2xs font-semibold whitespace-nowrap">
                    Type
                  </th>
                  <th class="text-left p-2xs font-semibold whitespace-nowrap">
                    Function
                  </th>
                  <th class="text-left p-2xs font-semibold whitespace-nowrap">
                    FuncArgs
                  </th>
                  <th class="text-left p-2xs font-semibold whitespace-nowrap">
                    Value
                  </th>
                </tr>
              </thead>
              <tbody>
                <template
                  v-for="attr in filteredAttributes"
                  :key="attr.attributeValueId"
                >
                  <!-- Main data row -->
                  <tr
                    class="border-b hover:bg-opacity-50"
                    :class="
                      themeClasses(
                        'border-neutral-100 hover:bg-neutral-50',
                        'border-neutral-800 hover:bg-neutral-800',
                      )
                    "
                  >
                    <!-- Toggle button -->
                    <td class="p-2xs">
                      <button
                        class="w-4 h-4 flex items-center justify-center rounded hover:bg-opacity-20 hover:bg-neutral-500"
                        :class="[
                          themeClasses('text-neutral-600', 'text-neutral-400'),
                          containsSearchMatch(attr) && searchQuery.trim()
                            ? themeClasses(
                                'bg-yellow-100 text-yellow-800',
                                'bg-yellow-800 text-yellow-200',
                              )
                            : '',
                        ]"
                        @click="
                          toggleExpandedRow('attr-' + attr.attributeValueId)
                        "
                      >
                        <span
                          class="text-xs transition-transform duration-200"
                          :class="
                            expandedRows.has('attr-' + attr.attributeValueId)
                              ? 'rotate-90'
                              : ''
                          "
                          >▶</span
                        >
                      </button>
                    </td>
                    <!-- Path column with truncation -->
                    <td
                      v-html-safe="highlight(stripRootFromPath(attr.path))"
                      class="p-2xs font-mono text-2xs max-w-32"
                      :title="attr.path"
                    ></td>
                    <td class="p-2xs">
                      <span
                        v-html-safe="highlight(attr.prop.kind)"
                        class="px-xs py-2xs rounded text-2xs"
                        :class="kindTagClass"
                      >
                      </span>
                    </td>
                    <td
                      v-html-safe="highlight(attr.funcName)"
                      class="p-2xs font-mono text-2xs opacity-70"
                    ></td>
                    <td class="p-2xs max-w-md">
                      <div
                        v-if="Object.keys(attr.funcArgs || {}).length"
                        class="font-mono text-2xs"
                      >
                        <div
                          v-if="getFuncArgsCount(attr.funcArgs) === 0"
                          class="text-2xs opacity-50"
                        >
                          none
                        </div>
                        <div
                          v-else-if="
                            isSmallContent(formatFuncArgs(attr.funcArgs))
                          "
                          v-html-safe="highlight(formatFuncArgs(attr.funcArgs))"
                          class="truncate"
                        ></div>
                        <details v-else class="cursor-pointer group">
                          <summary
                            class="hover:bg-opacity-20 hover:bg-neutral-500 rounded px-xs py-1 select-none outline-none list-none"
                          >
                            <div class="inline-flex items-center gap-xs">
                              <span
                                class="text-xs transition-transform duration-200 group-open:rotate-90"
                                >▶</span
                              >
                              <span
                                >{{ getFuncArgsCount(attr.funcArgs) }} args -
                                {{ getFuncArgsSummary(attr.funcArgs) }}</span
                              >
                            </div>
                          </summary>
                          <div class="mt-2xs">
                            <pre
                              v-html-safe="
                                highlight(formatFuncArgs(attr.funcArgs))
                              "
                              class="p-2xs rounded text-2xs overflow-auto max-h-32 whitespace-pre-wrap"
                              :class="docClass"
                            ></pre>
                          </div>
                        </details>
                      </div>
                      <span v-else class="text-2xs opacity-50">none</span>
                    </td>
                    <td class="p-2xs max-w-md">
                      <div
                        v-if="attr.value !== null"
                        class="font-mono text-2xs"
                      >
                        <div
                          v-if="isSmallContent(attr.value)"
                          v-html-safe="highlight(formatValue(attr.value))"
                          class="truncate"
                        ></div>
                        <details v-else class="cursor-pointer group">
                          <summary
                            class="hover:bg-opacity-20 hover:bg-neutral-500 rounded px-xs py-1 select-none outline-none list-none"
                          >
                            <div class="inline-flex items-center gap-xs">
                              <span
                                class="text-xs transition-transform duration-200 group-open:rotate-90"
                                >▶</span
                              >
                              <span
                                v-html-safe="
                                  highlight(getValueSummary(attr.value))
                                "
                              ></span>
                            </div>
                          </summary>
                          <div class="mt-2xs">
                            <pre
                              v-html-safe="highlight(formatValue(attr.value))"
                              class="p-2xs rounded text-2xs overflow-auto max-h-32 whitespace-pre-wrap"
                              :class="docClass"
                            ></pre>
                          </div>
                        </details>
                      </div>
                      <span v-else class="text-2xs opacity-50">null</span>
                    </td>
                  </tr>
                  <!-- Expanded raw data row -->
                  <tr
                    v-if="expandedRows.has('attr-' + attr.attributeValueId)"
                    class="border-b"
                    :class="
                      themeClasses(
                        'bg-neutral-50 border-neutral-100',
                        'bg-neutral-800 border-neutral-700',
                      )
                    "
                  >
                    <td colspan="6" class="p-sm">
                      <div class="rounded" :class="docClass">
                        <div class="text-sm font-semibold mb-xs opacity-80">
                          Raw Attribute Data
                        </div>
                        <pre
                          v-html-safe="highlight(JSON.stringify(attr, null, 2))"
                          class="text-xs overflow-auto max-h-96 whitespace-pre-wrap"
                        ></pre>
                      </div>
                    </td>
                  </tr>
                </template>
              </tbody>
            </table>
          </div>
          <div v-else class="text-center py-sm opacity-60">
            No attributes found
          </div>
        </div>

        <!-- Sockets Table -->
        <div
          v-if="filteredInputSockets.length || filteredOutputSockets.length"
          class="border rounded p-xs"
          :class="cardClass"
        >
          <h3 class="text-base font-semibold mb-xs" :class="headerClass">
            Sockets ({{
              filteredInputSockets.length + filteredOutputSockets.length
            }})
          </h3>
          <div class="overflow-x-auto">
            <table class="min-w-full text-xs">
              <thead>
                <tr
                  class="border-b"
                  :class="
                    themeClasses('border-neutral-200', 'border-neutral-700')
                  "
                >
                  <th class="text-left p-2xs font-semibold whitespace-nowrap">
                    Type
                  </th>
                  <th class="text-left p-2xs font-semibold whitespace-nowrap">
                    Name
                  </th>
                  <th class="text-left p-2xs font-semibold whitespace-nowrap">
                    Annotations
                  </th>
                  <th class="text-left p-2xs font-semibold whitespace-nowrap">
                    Value
                  </th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="socket in allSockets"
                  :key="socket.socketId"
                  class="border-b hover:bg-opacity-50"
                  :class="
                    themeClasses(
                      'border-neutral-100 hover:bg-neutral-50',
                      'border-neutral-800 hover:bg-neutral-800',
                    )
                  "
                >
                  <td class="p-2xs">
                    <span
                      class="px-xs py-2xs rounded text-2xs font-semibold"
                      :class="
                        socket.type === 'input'
                          ? themeClasses(
                              'bg-green-100 text-green-800',
                              'bg-green-800 text-green-200',
                            )
                          : themeClasses(
                              'bg-blue-100 text-blue-800',
                              'bg-blue-800 text-blue-200',
                            )
                      "
                    >
                      {{ socket.type }}
                    </span>
                  </td>
                  <td
                    v-html-safe="highlight(socket.name)"
                    class="p-2xs font-semibold"
                  ></td>
                  <td class="p-2xs max-w-md">
                    <div
                      v-if="socket.connectionAnnotations?.length"
                      class="flex flex-wrap gap-xs"
                    >
                      <span
                        v-for="annotation in socket.connectionAnnotations.slice(
                          0,
                          3,
                        )"
                        :key="annotation"
                        v-html-safe="highlight(annotation)"
                        class="px-xs py-2xs rounded text-2xs"
                        :class="tagClass"
                      >
                      </span>
                      <span
                        v-if="socket.connectionAnnotations.length > 3"
                        class="px-xs py-2xs rounded text-2xs opacity-60"
                        :class="tagClass"
                      >
                        +{{ socket.connectionAnnotations.length - 3 }} more
                      </span>
                    </div>
                    <span v-else class="text-2xs opacity-50">none</span>
                  </td>
                  <td class="p-2xs max-w-md">
                    <div
                      v-if="socket.value !== null"
                      class="font-mono text-2xs"
                    >
                      <div
                        v-if="isSmallContent(socket.value)"
                        v-html-safe="highlight(formatValue(socket.value))"
                        class="truncate"
                      ></div>
                      <details v-else class="cursor-pointer group">
                        <summary
                          class="hover:bg-opacity-20 hover:bg-neutral-500 rounded px-xs py-1 select-none outline-none list-none"
                        >
                          <div class="inline-flex items-center gap-xs">
                            <span
                              class="text-xs transition-transform duration-200 group-open:rotate-90"
                              >▶</span
                            >
                            <span
                              v-html-safe="
                                highlight(getValueSummary(socket.value))
                              "
                            ></span>
                          </div>
                        </summary>
                        <div class="mt-2xs">
                          <pre
                            v-html-safe="highlight(formatValue(socket.value))"
                            class="p-2xs rounded text-2xs overflow-auto max-h-32 whitespace-pre-wrap"
                            :class="docClass"
                          ></pre>
                        </div>
                      </details>
                    </div>
                    <span v-else class="text-2xs opacity-50">null</span>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>

      <EmptyState v-else text="No debug data available" icon="beaker" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import { useQuery } from "@tanstack/vue-query";
import { SiSearch, themeClasses } from "@si/vue-lib/design-system";
import { ComponentId } from "@/api/sdf/dal/component";
import { routes, useApi } from "./api_composables";
import EmptyState from "./EmptyState.vue";

interface ComponentDebugView {
  name: string;
  schemaVariantId: string;
  attributes: AttributeDebugView[];
  inputSockets: SocketDebugView[];
  outputSockets: SocketDebugView[];
  parentId: string | null;
  geometry: Record<string, GeometryInfo>;
}

interface AttributeDebugView {
  path: string;
  parentId: string | null;
  attributeValueId: string;
  funcId: string;
  valueIsFor: ValueIsFor;
  prop: PropInfo;
  prototypeId: string;
  prototypeIsComponentSpecific: boolean;
  key: string | null;
  funcName: string;
  funcArgs: Record<string, FuncArgDebugView[]>;
  value: unknown;
  propKind: string;
  view: unknown;
}

interface SocketDebugView extends AttributeDebugView {
  socketId: string;
  connectionAnnotations: string[];
  inferredConnections: string[];
  name: string;
}

interface PropInfo {
  id: string;
  created_at: string;
  updated_at: string;
  name: string;
  kind: string;
  widget_kind: string;
  widget_options: WidgetOption[] | null;
  doc_link: string | null;
  documentation: string | null;
  hidden: boolean;
  refers_to_prop_id: string | null;
  diff_func_id: string | null;
  validation_format: string | null;
  can_be_used_as_prototype_arg: boolean;
  ui_optionals: Record<string, unknown>;
}

interface ValueIsFor {
  kind: string;
  id: string;
}

interface FuncArgDebugView {
  value: unknown;
  name: string;
  valueSource: string;
  valueSourceId: string;
  socketSourceKind: string | null;
  path: string | null;
  isUsed: boolean;
}

interface WidgetOption {
  label: string;
  value: string;
}

interface GeometryInfo {
  id: string;
  created_at: string;
  updated_at: string;
  x: number;
  y: number;
  width: number;
  height: number;
}

const props = defineProps<{
  componentId: ComponentId;
}>();

const debugApi = useApi();

const componentDebugQuery = useQuery({
  queryKey: computed(() => ["component-debug", props.componentId]),
  staleTime: 60 * 1 * 1000,
  queryFn: async () => {
    const call = debugApi.endpoint<ComponentDebugView>(routes.ComponentDebug, {
      id: props.componentId,
    });
    const response = await call.get();
    return response.data;
  },
});

const componentData = computed(
  () => componentDebugQuery.data.value as ComponentDebugView | undefined,
);

const searchQuery = ref("");
const expandedRows = ref(new Set<string>());

// Style classes
const cardClass = computed(() =>
  themeClasses(
    "bg-shade-0 border-neutral-300",
    "bg-shade-100 border-neutral-600",
  ),
);
const headerClass = computed(() =>
  themeClasses("text-neutral-900", "text-neutral-100"),
);
const docClass = computed(() =>
  themeClasses(
    "bg-neutral-50 text-neutral-700",
    "bg-neutral-800 text-neutral-300",
  ),
);
const kindTagClass = computed(() =>
  themeClasses(
    "bg-purple-100 text-purple-800",
    "bg-purple-800 text-purple-200",
  ),
);
const destructiveClass = computed(() =>
  themeClasses("text-destructive-500", "text-destructive-400"),
);
const tagClass = computed(() =>
  themeClasses(
    "bg-neutral-100 text-neutral-700",
    "bg-neutral-700 text-neutral-300",
  ),
);
const bgClass = computed(() => themeClasses("bg-white", "bg-neutral-900"));

// Utility functions
const stripRootFromPath = (path: string): string => {
  const stripped = path.startsWith("root/") ? path.slice(5) : path;
  // Truncate from beginning if too long, keeping the end which is usually more specific
  if (stripped.length > 25) {
    return `...${stripped.slice(-22)}`;
  }
  return stripped;
};

const formatValue = (value: unknown): string => {
  if (value === null || value === undefined) return "null";
  return typeof value === "object"
    ? JSON.stringify(value, null, 2)
    : String(value);
};

const isSmallContent = (value: unknown): boolean => {
  if (value === null || value === undefined) return true;
  const formatted = formatValue(value);
  const lines = formatted.split("\n").length;
  if (["{}", "[]", "null", ""].includes(formatted.trim())) return true;
  return lines === 1 && formatted.length <= 60;
};

const getValueSummary = (value: unknown): string => {
  if (value === null || value === undefined) return "null";
  const formatted = formatValue(value);
  const lines = formatted.split("\n");
  if (lines.length === 1) {
    return formatted.length > 60
      ? `${formatted.substring(0, 57)}...`
      : formatted;
  }
  return `${lines.length} lines, ${formatted.length} chars`;
};

const getFuncArgsCount = (
  funcArgs: Record<string, FuncArgDebugView[]>,
): number => {
  return Object.values(funcArgs || {}).reduce(
    (total, args) => total + args.length,
    0,
  );
};

const getFuncArgsSummary = (
  funcArgs: Record<string, FuncArgDebugView[]>,
): string => {
  const allArgs: string[] = [];
  Object.entries(funcArgs || {}).forEach(([argName, args]) => {
    args.forEach((arg) => {
      const value = arg.value !== null ? formatValue(arg.value) : "null";
      const shortValue =
        value.length > 20 ? `${value.substring(0, 17)}...` : value;
      allArgs.push(`${argName}: ${shortValue}`);
    });
  });
  return allArgs.slice(0, 2).join(", ");
};

const formatFuncArgs = (
  funcArgs: Record<string, FuncArgDebugView[]>,
): string => {
  const sections: string[] = [];
  Object.entries(funcArgs || {}).forEach(([argName, args]) => {
    args.forEach((arg, index) => {
      const section = [
        `[${argName}${args.length > 1 ? ` #${index + 1}` : ""}]`,
        `Source: ${arg.valueSource}`,
        arg.path ? `Path: ${arg.path}` : null,
        `Used: ${arg.isUsed ? "Yes" : "No"}`,
        `Value: ${formatValue(arg.value)}`,
      ]
        .filter(Boolean)
        .join("\n");
      sections.push(section);
    });
  });
  return sections.join("\n\n");
};

const containsSearchMatch = (content: unknown): boolean => {
  if (!searchQuery.value.trim()) return false;
  const searchText =
    typeof content === "string" ? content : JSON.stringify(content);
  return searchText.toLowerCase().includes(searchQuery.value.toLowerCase());
};

const highlightMatch = (text: string, query: string): string => {
  if (!query.trim()) return text;
  const escapedQuery = query.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const regex = new RegExp(`(${escapedQuery})`, "gi");
  return text.replace(
    regex,
    '<mark class="bg-yellow-200 dark:bg-yellow-600 px-1 rounded">$1</mark>',
  );
};

const highlight = (text: string): string =>
  searchQuery.value.trim() ? highlightMatch(text, searchQuery.value) : text;

const toggleExpandedRow = (rowId: string) => {
  if (expandedRows.value.has(rowId)) {
    expandedRows.value.delete(rowId);
  } else {
    expandedRows.value.add(rowId);
  }
};

// Filtered data
const filteredAttributes = computed(() => {
  if (!componentData.value?.attributes) return [];
  if (!searchQuery.value.trim()) return componentData.value.attributes;

  return componentData.value.attributes.filter((attr) =>
    containsSearchMatch(attr),
  );
});

const filteredInputSockets = computed(() => {
  if (!componentData.value?.inputSockets) return [];
  if (!searchQuery.value.trim()) return componentData.value.inputSockets;

  return componentData.value.inputSockets.filter((socket) =>
    containsSearchMatch(socket),
  );
});

const filteredOutputSockets = computed(() => {
  if (!componentData.value?.outputSockets) return [];
  if (!searchQuery.value.trim()) return componentData.value.outputSockets;

  return componentData.value.outputSockets.filter((socket) =>
    containsSearchMatch(socket),
  );
});

// Combined sockets for table display
const allSockets = computed(() => [
  ...filteredInputSockets.value.map((socket) => ({
    ...socket,
    type: "input" as const,
  })),
  ...filteredOutputSockets.value.map((socket) => ({
    ...socket,
    type: "output" as const,
  })),
]);
</script>
