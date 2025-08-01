<template>
  <div class="history-panel">
    <div v-if="isLoading" class="flex items-center justify-center p-md">
      <div class="text-sm text-neutral-600 dark:text-neutral-400">
        Loading history...
      </div>
    </div>
    <div v-else-if="error" class="flex items-center justify-center p-md">
      <div class="text-sm text-red-600 dark:text-red-400">
        Error loading audit logs: {{ error }}
      </div>
    </div>
    <div v-else-if="!auditLogs.length" class="flex items-center justify-center p-md">
      <div class="text-sm text-neutral-600 dark:text-neutral-400">
        No history available for this component
      </div>
    </div>
    <div v-else-if="!filteredAuditLogs.length" class="flex items-center justify-center p-md">
      <div class="text-sm text-neutral-600 dark:text-neutral-400">
        No history entries match the current filters
      </div>
    </div>
    <div v-else class="flex flex-col gap-xs p-xs">
    <!-- Filter Controls -->
    <div class="flex flex-col gap-xs p-xs border-b border-neutral-200 dark:border-neutral-700 mb-xs">
      <div class="flex flex-col gap-xs">
        <label class="flex items-center gap-xs text-xs text-neutral-600 dark:text-neutral-400 cursor-pointer">
          <input
            v-model="showCurrentChangesetOnly"
            type="checkbox"
            class="rounded border-neutral-300 dark:border-neutral-600 text-blue-600 focus:ring-blue-500 focus:ring-2"
          />
          This Change Set Only
        </label>
        <label class="flex items-center gap-xs text-xs text-neutral-600 dark:text-neutral-400 cursor-pointer">
          <input
            v-model="hideSystemChanges"
            type="checkbox"
            class="rounded border-neutral-300 dark:border-neutral-600 text-blue-600 focus:ring-blue-500 focus:ring-2"
          />
          Hide system changes
        </label>
      </div>
      <div class="flex items-center justify-between">
        <span class="text-xs text-neutral-500 dark:text-neutral-500">
          ({{ filteredAuditLogs.length }} of {{ auditLogs.length }} entries{{ canLoadMore ? ', more available' : '' }})
        </span>
        <button
          v-if="showCurrentChangesetOnly"
          @click="showCurrentChangesetOnly = false"
          class="text-xs text-blue-600 dark:text-blue-400 hover:underline"
        >
          View all changes
        </button>
      </div>
    </div>

    <div class="relative">
      <!-- Continuous timeline line -->
      <div class="absolute left-4 top-0 bottom-0 w-0.5 bg-neutral-300 dark:bg-neutral-600"></div>
      
      <div
        v-for="log in filteredAuditLogs"
        :key="`${log.timestamp}-${log.kind}`"
        class="relative flex items-start cursor-pointer hover:bg-neutral-50 dark:hover:bg-neutral-800/50 transition-colors rounded-sm py-2 pl-2"
        @click="(event) => handleLogClick(log, event)"
      >
        <!-- Commit dot -->
        <div class="absolute left-2.5 top-3 z-10">
          <div 
            class="w-3 h-3 rounded-full border-2 border-white dark:border-neutral-900"
            :style="{ backgroundColor: getCommitDotColor(log) }"
          ></div>
        </div>

        <!-- Commit content -->
        <div class="flex-1 min-w-0 ml-6">
          <div class="flex items-center justify-between mb-1">
            <div class="flex items-center gap-xs">
              <span class="text-sm font-medium text-neutral-900 dark:text-neutral-100">
                {{ log.title }}
              </span>
              <template v-if="hasEnhancedDetails(log)">
                <Icon
                  :name="expandedLogs.has(getLogKey(log)) ? 'chevron-down' : 'chevron-right'"
                  size="xs"
                  class="text-neutral-400"
                />
              </template>
            </div>
          </div>
          
          <div class="text-xs text-neutral-500 dark:text-neutral-400 mb-1">
            {{ formatTimestampUTC(log.timestamp) }}
          </div>
          
          <div class="text-xs text-neutral-600 dark:text-neutral-400">
            <span class="font-medium">Changed By:</span> {{ log.userEmail || log.userName || "System" }}
          </div>

          <!-- Enhanced details for specific log types -->
          <div v-if="hasEnhancedDetails(log)" class="text-xs text-neutral-600 dark:text-neutral-400 mt-2">
            <div v-if="getMetadataDetails(log).length > 0" class="flex flex-wrap gap-x-md gap-y-xs mb-xs">
              <span
                v-for="detail in getMetadataDetails(log)"
                :key="detail"
                class="bg-neutral-200 dark:bg-neutral-700 px-xs py-2xs rounded"
              >
                {{ detail }}
              </span>
            </div>
            
            <!-- Missing attribute notification -->
            <div v-if="isAttributeMissing(log)" class="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded p-xs mb-xs">
              <div class="flex items-center gap-xs">
                <span class="text-yellow-600 dark:text-yellow-400">⚠️</span>
                <div class="text-yellow-800 dark:text-yellow-200">
                  <div class="font-medium">Attribute not found in current tree</div>
                  <div class="text-xs opacity-75">This attribute may have been deleted or restructured</div>
                </div>
              </div>
            </div>
            
            <!-- Before/After Values -->
            <div v-if="hasBeforeAfterValues(log)" class="space-y-xs">
              <!-- Show operation type context -->
              <div v-if="isUnsetOperation(log)" class="text-xs text-orange-600 dark:text-orange-400 font-medium mb-xs">
                Property was unset/removed
              </div>
              <div v-else-if="isSetOperation(log)" class="text-xs text-green-600 dark:text-green-400 font-medium mb-xs">
                Property was previously empty
              </div>
              <div v-else class="text-xs text-blue-600 dark:text-blue-400 font-medium mb-xs">
                Property value changed
              </div>
              
              <!-- Before value - show if we have meaningful before data (not for initial set operations) -->
              <div v-if="!isSetOperation(log) && (getBeforeValue(log) !== undefined || isUnsetOperation(log))" class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded p-xs">
                <div class="font-medium text-red-700 dark:text-red-300 mb-2xs">Before:</div>
                <pre class="text-red-600 dark:text-red-400 whitespace-pre-wrap text-xs">{{ formatValue(getBeforeValue(log)) }}</pre>
              </div>
              
              <!-- After value - always show if we have after data or if it's a set operation -->
              <div v-if="getAfterValue(log) !== undefined || isSetOperation(log)" class="bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded p-xs">
                <div class="font-medium text-green-700 dark:text-green-300 mb-2xs">
                  {{ isUnsetOperation(log) ? 'After (unset):' : 'After:' }}
                </div>
                <pre class="text-green-600 dark:text-green-400 whitespace-pre-wrap text-xs">{{ formatValue(getAfterValue(log)) }}</pre>
              </div>
            </div>
          </div>

          <!-- Expanded details view -->
          <div v-if="expandedLogs.has(getLogKey(log))" class="mt-xs space-y-xs">
            <!-- System details formatted as key-value pairs -->
            <div class="text-xs bg-neutral-50 dark:bg-neutral-800 rounded p-xs">
              <div class="grid grid-cols-1 gap-1">
                <div><span class="font-medium text-neutral-700 dark:text-neutral-300">User:</span> <span class="text-neutral-600 dark:text-neutral-400">{{ log.userName || "System" }}</span></div>
                <div><span class="font-medium text-neutral-700 dark:text-neutral-300">Email:</span> <span class="text-neutral-600 dark:text-neutral-400">{{ log.userEmail || "N/A" }}</span></div>
                <div><span class="font-medium text-neutral-700 dark:text-neutral-300">Action:</span> <span class="text-neutral-600 dark:text-neutral-400">{{ log.kind }}</span></div>
                <div v-if="log.entityName"><span class="font-medium text-neutral-700 dark:text-neutral-300">Entity ID:</span> <span class="text-neutral-600 dark:text-neutral-400">{{ log.entityName }}</span></div>
                <div><span class="font-medium text-neutral-700 dark:text-neutral-300">Entity Type:</span> <span class="text-neutral-600 dark:text-neutral-400">{{ log.entityType }}</span></div>
                <div><span class="font-medium text-neutral-700 dark:text-neutral-300">Timestamp:</span> <span class="text-neutral-600 dark:text-neutral-400">{{ log.timestamp }}</span></div>
                <div v-if="log.changeSetId"><span class="font-medium text-neutral-700 dark:text-neutral-300">Change Set ID:</span> <span class="text-neutral-600 dark:text-neutral-400">{{ log.changeSetId }}</span></div>
                <div v-if="log.changeSetName"><span class="font-medium text-neutral-700 dark:text-neutral-300">Change Set Name:</span> <span class="text-neutral-600 dark:text-neutral-400">{{ log.changeSetName }}</span></div>
                <div v-if="log.authenticationMethod"><span class="font-medium text-neutral-700 dark:text-neutral-300">Auth Method:</span> <span class="text-neutral-600 dark:text-neutral-400">{{ log.authenticationMethod.method }}</span></div>
              </div>
            </div>
            
            <!-- Clickable JSON blob -->
            <div>
              <div class="text-xs font-medium text-neutral-700 dark:text-neutral-300 mb-xs">
                Full Audit Log Data:
              </div>
              <div 
                class="text-xs text-neutral-600 dark:text-neutral-400 bg-neutral-100 dark:bg-neutral-900 p-xs rounded cursor-pointer hover:bg-neutral-200 dark:hover:bg-neutral-800 transition-colors border border-transparent hover:border-neutral-300 dark:hover:border-neutral-600"
                @click.stop="showJsonModal = true; selectedLog = log"
                title="Click to view full JSON in modal"
              >
                <pre class="overflow-auto max-h-32 whitespace-pre-wrap">{{ JSON.stringify(log, null, 2) }}</pre>
                <div class="text-center mt-2 text-neutral-500 dark:text-neutral-400 text-xs">
                  → Click to expand in modal
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Show More Button -->
    <div v-if="canLoadMore" class="flex justify-center p-xs border-t border-neutral-200 dark:border-neutral-700">
      <button
        :disabled="isLoadingMore"
        :class="[
          'px-md py-xs text-sm rounded transition-colors',
          'bg-neutral-100 dark:bg-neutral-800',
          'border border-neutral-300 dark:border-neutral-600',
          'text-neutral-700 dark:text-neutral-300',
          'hover:bg-neutral-200 dark:hover:bg-neutral-700',
          'disabled:opacity-50 disabled:cursor-not-allowed'
        ]"
        @click="loadMore"
      >
        <span v-if="isLoadingMore" class="flex items-center gap-xs">
          <div class="w-3 h-3 border border-neutral-400 border-t-transparent rounded-full animate-spin"></div>
          Loading more...
        </span>
        <span v-else>Show More</span>
      </button>
    </div>
  </div>

  <!-- JSON Modal -->
  <div 
    v-if="showJsonModal && selectedLog" 
    class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4"
    @click="showJsonModal = false"
  >
    <div 
      class="bg-white dark:bg-neutral-900 rounded-lg shadow-xl max-w-4xl w-full max-h-[80vh] flex flex-col"
      @click.stop
    >
      <div class="flex items-center justify-between p-4 border-b border-neutral-200 dark:border-neutral-700">
        <h3 class="text-lg font-semibold text-neutral-900 dark:text-neutral-100">
          Full Audit Log Data
        </h3>
        <button 
          @click="showJsonModal = false"
          class="text-neutral-500 hover:text-neutral-700 dark:text-neutral-400 dark:hover:text-neutral-200"
        >
          <Icon name="x" size="sm" />
        </button>
      </div>
      <div class="flex-1 overflow-auto p-4">
        <pre class="text-sm text-neutral-700 dark:text-neutral-300 whitespace-pre-wrap">{{ JSON.stringify(selectedLog, null, 2) }}</pre>
      </div>
      <div class="flex justify-end gap-2 p-4 border-t border-neutral-200 dark:border-neutral-700">
        <button 
          @click="copyToClipboard(JSON.stringify(selectedLog, null, 2))"
          class="px-3 py-2 text-sm bg-neutral-100 dark:bg-neutral-800 text-neutral-700 dark:text-neutral-300 rounded hover:bg-neutral-200 dark:hover:bg-neutral-700 transition-colors"
        >
          Copy JSON
        </button>
        <button 
          @click="showJsonModal = false"
          class="px-3 py-2 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
        >
          Close
        </button>
      </div>
    </div>
    </div>

    <!-- JSON Modal -->
    <div 
      v-if="showJsonModal && selectedLog" 
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4"
      @click="showJsonModal = false"
    >
      <div 
        class="bg-white dark:bg-neutral-900 rounded-lg shadow-xl max-w-4xl w-full max-h-[80vh] flex flex-col"
        @click.stop
      >
        <div class="flex items-center justify-between p-4 border-b border-neutral-200 dark:border-neutral-700">
          <h3 class="text-lg font-semibold text-neutral-900 dark:text-neutral-100">
            Full Audit Log Data
          </h3>
          <button 
            @click="showJsonModal = false"
            class="text-neutral-500 hover:text-neutral-700 dark:text-neutral-400 dark:hover:text-neutral-200"
          >
            <Icon name="x" size="sm" />
          </button>
        </div>
        <div class="flex-1 overflow-auto p-4">
          <pre class="text-sm text-neutral-700 dark:text-neutral-300 whitespace-pre-wrap">{{ JSON.stringify(selectedLog, null, 2) }}</pre>
        </div>
        <div class="flex justify-end gap-2 p-4 border-t border-neutral-200 dark:border-neutral-700">
          <button 
            @click="copyToClipboard(JSON.stringify(selectedLog, null, 2))"
            class="px-3 py-2 text-sm bg-neutral-100 dark:bg-neutral-800 text-neutral-700 dark:text-neutral-300 rounded hover:bg-neutral-200 dark:hover:bg-neutral-700 transition-colors"
          >
            Copy JSON
          </button>
          <button 
            @click="showJsonModal = false"
            class="px-3 py-2 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, computed, inject } from "vue";
import { Icon } from "@si/vue-lib/design-system";
import { BifrostComponent } from "@/workers/types/entity_kind_types";
import { useApi, routes } from "./api_composables";
import { Context, assertIsDefined } from "./types";
import { useAttributeHighlight, extractPropertyPathFromAuditLog } from "./logic_composables/attribute_highlight";

const props = defineProps<{
  component: BifrostComponent;
}>();

// Get current changeset context
const ctx = inject<Context>("CONTEXT");
assertIsDefined(ctx);
const currentChangesetId = computed(() => ctx?.changeSetId.value);

interface AuditLog {
  title: string;
  userId?: string;
  userEmail?: string;
  userName?: string;
  kind: string;
  entityName: string;
  entityType: string;
  timestamp: string;
  changeSetId?: string;
  changeSetName?: string;
  metadata: Record<string, unknown>;
  authenticationMethod: {
    method: string;
    role?: string;
    tokenId?: string;
  };
}

// Reactive state
const api = useApi();
const auditLogs = ref<AuditLog[]>([]);
const isLoading = ref(false);
const isLoadingMore = ref(false);
const error = ref<string | null>(null);
const expandedLogs = ref<Set<string>>(new Set());
const hideSystemChanges = ref(true);
const canLoadMore = ref(false);
const currentSize = ref(50); // Start with more entries
const showCurrentChangesetOnly = ref(true); // Default to current changeset only
const showJsonModal = ref(false);
const selectedLog = ref<AuditLog | null>(null);

// Attribute highlighting
const { highlightAttribute, clearHighlight, getCurrentExpandedLogKey } = useAttributeHighlight();
const currentExpandedLogKey = getCurrentExpandedLogKey();

// Helper function
function isSystemChange(log: AuditLog): boolean {
  if (!log.userName || log.userName === "System") return true;
  
  if (log.authenticationMethod?.method === "System" || 
      log.authenticationMethod?.method === "Automatic") return true;
  
  const systemChangeTypes = [
    "SystemUpdate",
    "AutomaticValidation", 
    "BackgroundProcess", 
    "SystemGenerated"
  ];
  
  return systemChangeTypes.includes(log.kind);
}

// Computed property
const filteredAuditLogs = computed<AuditLog[]>(() => {
  let filtered = auditLogs.value;
  
  // Filter by changeset if enabled
  if (showCurrentChangesetOnly.value && currentChangesetId.value) {
    filtered = filtered.filter(log => log.changeSetId === currentChangesetId.value);
  }
  
  // Filter system changes if enabled
  if (hideSystemChanges.value) {
    filtered = filtered.filter(log => !isSystemChange(log));
  }
  
  return filtered;
});

const loadAuditLogs = async (append = false) => {
  if (!props.component.id) return;
  
  if (append) {
    isLoadingMore.value = true;
  } else {
    isLoading.value = true;
    auditLogs.value = []; // Clear existing logs for fresh load
  }
  error.value = null;
  
  try {
    const response = await api
      .endpoint<{ logs: AuditLog[]; canLoadMore: boolean }>(routes.AuditLogs)
      .get(new URLSearchParams({
        component_id: props.component.id,
        size: currentSize.value.toString(),
        sort_ascending: "false",
      }));

    if (api.ok(response)) {
      if (append) {
        // Append new logs to existing ones
        auditLogs.value = [...auditLogs.value, ...response.data.logs];
      } else {
        // Replace logs completely
        auditLogs.value = response.data.logs;
      }
      canLoadMore.value = response.data.canLoadMore;
    } else {
      error.value = "Failed to load audit logs";
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Unknown error";
  } finally {
    isLoading.value = false;
    isLoadingMore.value = false;
  }
};

const refreshHistory = (): void => {
  loadAuditLogs();
};

const loadMore = (): void => {
  if (canLoadMore.value && !isLoadingMore.value) {
    currentSize.value += 25; // Load 25 more entries
    loadAuditLogs(true);
  }
};

// Function to collapse the currently expanded log entry
const collapseExpandedLog = () => {
  if (currentExpandedLogKey.value) {
    expandedLogs.value.delete(currentExpandedLogKey.value);
  }
};

// Expose refresh function and collapse function for parent component
defineExpose({
  refresh: refreshHistory,
  collapseExpandedLog,
});

const formatTimestamp = (timestamp: string): string => {
  const date = new Date(timestamp);
  const now = new Date();
  const diffInHours = (now.getTime() - date.getTime()) / (1000 * 60 * 60);
  
  if (diffInHours < 1) {
    const diffInMinutes = Math.floor(diffInHours * 60);
    return `${diffInMinutes}m ago`;
  } else if (diffInHours < 24) {
    return `${Math.floor(diffInHours)}h ago`;
  } else {
    const diffInDays = Math.floor(diffInHours / 24);
    return `${diffInDays}d ago`;
  }
};

const formatTimestampUTC = (timestamp: string): string => {
  const date = new Date(timestamp);
  return date.toISOString().replace('T', ' ').replace(/\.\d{3}Z$/, ' UTC');
};

const getLogKey = (log: AuditLog): string => {
  return `${log.timestamp}-${log.kind}-${log.entityName}`;
};

const toggleLogExpansion = (log: AuditLog): void => {
  const key = getLogKey(log);
  const newExpanded = new Set(expandedLogs.value);
  if (newExpanded.has(key)) {
    newExpanded.delete(key);  
  } else {
    newExpanded.add(key);
  }
  expandedLogs.value = newExpanded;
};

const hasEnhancedDetails = (log: AuditLog): boolean => {
  return log.metadata && (
    log.kind === 'SetDependent' ||
    hasBeforeAfterValues(log) ||
    getMetadataDetails(log).length > 0
  );
};

const hasBeforeAfterValues = (log: AuditLog): boolean => {
  return log.metadata && (
    log.metadata.beforeValue !== undefined ||
    log.metadata.afterValue !== undefined
  );
};

const isUnsetOperation = (log: AuditLog): boolean => {
  return log.metadata && (
    (log.metadata.beforeValue !== undefined && log.metadata.beforeValue !== null && log.metadata.beforeValue !== '') &&
    (log.metadata.afterValue === undefined || log.metadata.afterValue === null || log.metadata.afterValue === '')
  );
};

const isSetOperation = (log: AuditLog): boolean => {
  return log.metadata && (
    (log.metadata.beforeValue === undefined || log.metadata.beforeValue === null || log.metadata.beforeValue === '') &&
    (log.metadata.afterValue !== undefined && log.metadata.afterValue !== null && log.metadata.afterValue !== '')
  );
};

const getBeforeValue = (log: AuditLog): unknown => {
  return log.metadata?.beforeValue;
};

const getAfterValue = (log: AuditLog): unknown => {
  return log.metadata?.afterValue;
};

const formatValue = (value: unknown): string => {
  if (value === null) return '(empty)';
  if (value === undefined) return '(empty)';
  if (typeof value === 'string') {
    if (value === '') return '(empty)';
    return value;
  }
  if (typeof value === 'number' || typeof value === 'boolean') return String(value);
  
  // For arrays, check if empty
  if (Array.isArray(value)) {
    if (value.length === 0) return '(empty array)';
    try {
      return JSON.stringify(value, null, 2);
    } catch {
      return String(value);
    }
  }
  
  // For objects, check if empty
  if (typeof value === 'object' && value !== null) {
    if (Object.keys(value).length === 0) return '(empty object)';
    try {
      return JSON.stringify(value, null, 2);
    } catch {
      return String(value);
    }
  }
  
  // Fallback
  return String(value);
};

const getCommitDotColor = (log: AuditLog): string => {
  // Color code different types of operations like git
  if (isUnsetOperation(log)) {
    return '#ef4444'; // Deletions in red
  } else if (isSetOperation(log)) {
    return '#22c55e'; // Additions in green
  } else if (log.kind === 'CreateComponent' || log.kind === 'CreateConnection') {
    return '#3b82f6'; // Creates in blue
  } else if (log.kind === 'DeleteComponent' || log.kind === 'DeleteConnection') {
    return '#ef4444'; // Deletes in red
  } else if (log.kind.startsWith('Update') || log.kind === 'Updated Component') {
    return '#eab308'; // Updates in yellow
  } else {
    return '#9ca3af'; // Default
  }
};

const currentChangesetName = computed(() => {
  // Get the changeset name from context
  if (!ctx?.changeSet?.value) return '';
  return ctx.changeSet.value.name || '';
});

const getMetadataDetails = (log: AuditLog): string[] => {
  const details: string[] = [];
  
  if (log.metadata) {
    // Handle UpdatePropertyEditorValue - show property path prominently
    if (log.kind === 'UpdatePropertyEditorValue' || log.kind === 'Updated Component') {
      // Try to get a meaningful property path from different possible metadata fields
      const propertyPath = log.metadata.attributePath || 
                          log.metadata.propertyPath || 
                          log.metadata.propName ||
                          log.metadata.propertyName;
      
      if (propertyPath) {
        details.push(`Property Path: ${propertyPath}`);
      }
      
      // Also show the property name if different from path
      if (log.metadata.propName && log.metadata.propName !== propertyPath) {
        details.push(`Property: ${log.metadata.propName}`);
      }
    }
    
    // Handle SetDependent specific details
    if (log.kind === 'SetDependent') {
      if (log.metadata.dependencyKind) {
        details.push(`Type: ${log.metadata.dependencyKind}`);
      }
      if (log.metadata.propertyPath) {
        details.push(`Property Path: ${log.metadata.propertyPath}`);
      }
      if (log.metadata.dependentValueId) {
        details.push(`Value ID: ${log.metadata.dependentValueId}`);
      }
      if (log.metadata.dependsOnValueId) {
        details.push(`Depends on: ${log.metadata.dependsOnValueId}`);
      }
      if (log.metadata.componentId && log.metadata.componentId !== props.component.id) {
        details.push(`Related Component: ${log.metadata.componentId}`);
      }
    }
    
    // Handle other common metadata fields
    if (log.metadata.attributePath && !details.some(d => d.includes('Property Path'))) {
      details.push(`Attribute Path: ${log.metadata.attributePath}`);
    }
    if (log.metadata.actionKind) {
      details.push(`Action: ${log.metadata.actionKind}`);
    }
    
    // Add schema variant info if available for context
    if (log.metadata.schemaVariantDisplayName && log.metadata.schemaVariantDisplayName !== props.component.schemaVariantName) {
      details.push(`Schema: ${log.metadata.schemaVariantDisplayName}`);
    }
  }
  
  return details;
};

const handleLogClick = (log: AuditLog, event: MouseEvent) => {
  console.log('👀 Audit log clicked:', log);
  
  // Check if the click is on the JSON blob area (prevent double handling)
  const target = event.target as HTMLElement;
  if (target.closest('[title="Click to view full JSON in modal"]')) {
    return; // Let the JSON modal handle this
  }
  
  const logKey = getLogKey(log);
  const wasExpanded = expandedLogs.value.has(logKey);
  
  console.log('🔑 Log key:', logKey, 'Was expanded:', wasExpanded);
  
  // Toggle expansion
  toggleLogExpansion(log);
  
  // Handle highlighting based on expansion state
  const propertyPath = extractPropertyPathFromAuditLog(log);
  console.log('🗺 Extracted property path:', propertyPath);
  
  if (propertyPath && (log.kind === 'UpdatePropertyEditorValue' || log.kind === 'Updated Component')) {
    if (wasExpanded) {
      // Was expanded, now collapsed - clear highlight
      console.log('🔴 Clearing highlight for:', propertyPath);
      clearHighlight(propertyPath);
    } else {
      // Was collapsed, now expanded - highlight attribute with change data
      const changeData = {
        beforeValue: log.metadata?.beforeValue,
        afterValue: log.metadata?.afterValue,
        operation: log.kind === 'UpdatePropertyEditorValue' ? 'update' : 'change'
      };
      console.log('🔴 Highlighting attribute:', propertyPath, 'with data:', changeData);
      highlightAttribute(propertyPath, changeData, undefined, logKey); // Persistent highlight until user changes
    }
  } else {
    console.log('🙅 Not highlighting - path:', propertyPath, 'kind:', log.kind);
  }
};

const copyToClipboard = (text: string) => {
  navigator.clipboard.writeText(text).then(() => {
    // Could add a toast notification here if available
    console.log('JSON copied to clipboard');
  }).catch(err => {
    console.error('Failed to copy JSON:', err);
  });
};

// Check if an attribute exists in the current DOM
const isAttributeMissing = (log: AuditLog): boolean => {
  const propertyPath = extractPropertyPathFromAuditLog(log);
  if (!propertyPath) return false;
  
  // For recently created attributes, be more lenient and don't show missing immediately
  // Check if this is a recent log entry (within last 5 seconds)
  const logTime = new Date(log.timestamp).getTime();
  const now = new Date().getTime();
  const isRecent = (now - logTime) < 5000; // 5 seconds
  
  if (isRecent) {
    // For recent entries, be more patient and don't show as missing immediately
    console.log('🕒 Recent audit log entry, being patient with DOM updates:', propertyPath);
    return false;
  }
  
  // Check if element exists in DOM using the same advanced matching as the highlighting system
  const allElements = document.querySelectorAll('[data-attribute-path]');
  
  // First try exact match
  if (document.querySelector(`[data-attribute-path="${propertyPath}"]`)) {
    return false;
  }
  
  // Try alternative path formats  
  const alternativePaths = [
    propertyPath.replace('/domain/', 'root/domain/'),
    propertyPath.replace('/domain', '/root/domain'),
  ];
  
  for (const altPath of alternativePaths) {
    if (document.querySelector(`[data-attribute-path="${altPath}"]`)) {
      return false;
    }
  }
  
  // Use the same advanced matching logic as the highlighting system
  const hasMatch = Array.from(allElements).some(el => {
    const elementPath = el.getAttribute('data-attribute-path');
    if (!elementPath) return false;
    
    // Exact match
    if (elementPath === propertyPath) return true;
    
    // Try semantic matching for array formats
    // This will match /domain/SsmAssociations/0/Key with /domain/SsmAssociations/SsmAssociationsItem[0]/Key
    return doPathsMatchForMissingCheck(elementPath, propertyPath);
  });
  
  return !hasMatch;
};

// Simplified version of path matching for missing attribute check
const doPathsMatchForMissingCheck = (domPath: string, auditPath: string): boolean => {
  // Normalize both paths to compare semantically
  const normalizePath = (path: string) => {
    return path
      // Convert Item[0] format to /0/ format
      .replace(/([A-Za-z]+)Item\[(\d+)\]/g, '/$2/')
      // Normalize multiple slashes
      .replace(/\/+/g, '/')
      // Remove trailing slash
      .replace(/\/$/, '');
  };
  
  const normalizedDomPath = normalizePath(domPath);
  const normalizedAuditPath = normalizePath(auditPath);
  
  return normalizedDomPath === normalizedAuditPath;
};

onMounted(() => {
  loadAuditLogs();
});
</script>