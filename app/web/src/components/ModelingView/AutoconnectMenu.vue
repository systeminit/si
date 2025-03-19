<template>
  <Modal
    ref="modalRef"
    title="Refactor Component Connections"
    size="2xl"
    noAutoFocus
    class="max-w-[75vw] overflow-hidden"
  >
    <div
      v-if="autoconnectData"
      class="mx-auto my-4 w-full max-w-3xl flex flex-col"
      style="max-height: 70vh"
    >
      <!-- HEADER -->
      <div class="mb-sm flex-col flex">
        <div v-if="createdConnections > 0" class="px-xs py-2xs">
          Created {{ createdConnections }} connections for
          {{ autoconnectData.componentName }}
        </div>
        <div v-else class="px-xs py-2xs">
          Couldn't automatically create any connections for
          {{ autoconnectData.componentName }}.
        </div>
        <div
          v-if="(autoconnectData?.potentialConnections?.length ?? 0) > 0"
          class="px-xs py-2xs"
        >
          Here are the potential connections found that match. For each manually
          set value you wish to replace, choose the correct connection.
        </div>
      </div>

      <div class="flex-1 overflow-y-auto space-y-4">
        <div
          v-for="connection in autoconnectData?.potentialConnections ?? []"
          :key="connection.socketId"
          class="border dark:border-neutral-700 border-neutral-200 my-xs"
        >
          <div v-if="connection && connection.processingConnections.length > 0">
            <!-- Header for each socket with potential connections -->
            <div
              :class="
                clsx(
                  'px-3 py-2 cursor-pointer flex items-center justify-between',
                  {
                    'bg-warning-600':
                      selectedConnections[connection.socketId]?.state ===
                      'hasSelections',
                    'bg-neutral-100 dark:bg-neutral-800':
                      selectedConnections[connection.socketId]?.state ===
                      'empty',
                    'bg-success-600':
                      selectedConnections[connection.socketId]?.state ===
                      'satisfied',
                  },
                )
              "
              @click="toggleCollapse(connection.socketId)"
            >
              Input Socket: {{ connection.socketName }}
              <Icon
                :name="
                  expandedConnections.includes(connection.socketId)
                    ? 'chevron--down'
                    : 'chevron--right'
                "
                size="sm"
              />
            </div>
            <transition name="expand">
              <div
                v-show="expandedConnections.includes(connection.socketId)"
                class="p-3 space-y-4"
              >
                <!-- SINGLE-ARITY SOCKET-->

                <div v-if="connection.socketArity === 'one'">
                  <p class="text-sm text-gray-500 mb-1">
                    Click a row below to select exactly one match:
                  </p>
                  <div
                    class="grid grid-cols-3 gap-2 font-semibold border-b py-2"
                  >
                    <div>Component Name</div>
                    <div>Asset Name</div>
                    <div>Socket Name</div>
                  </div>
                  <div
                    v-for="match in connection.processingConnections"
                    :key="match.key"
                    :class="clsx(
                    'grid grid-cols-3 p-2 border-b last:border-b-0 cursor-pointer',
                    (selectedConnections[connection.socketId] as SingleConnectionState).socketId === match.key ? ['bg-action-100 dark:bg-action-700',] :
                      ['dark:hover:text-action-300 hover:text-action-500',
                        'bg-neutral-100 dark:bg-neutral-700 group'
                      ]
                  )"
                    @click="
                      selectSingleConnection(connection.socketId, match.key)
                    "
                  >
                    <div>{{ match.componentName }}</div>
                    <div>{{ match.schemaVariantName }}</div>
                    <div>
                      <TruncateWithTooltip>
                        {{ match.socketName }}
                      </TruncateWithTooltip>
                    </div>
                  </div>
                </div>

                <!-- MANY-ARITY SOCKET -->
                <div v-else>
                  <!-- Toggle for 'bulk' vs 'individual' if both are available -->
                  <div
                    v-if="
                      bulkMatches(connection).length > 0 &&
                      canDoIndividual(connection)
                    "
                    class="flex space-x-4 mb-2"
                  >
                    <label class="inline-flex items-center">
                      <input
                        v-model="(selectedConnections[connection.socketId] as ManyConnectionState).mode"
                        type="radio"
                        value="bulk"
                        class="mr-1"
                      />
                      Bulk (one connection for entire array)
                    </label>
                    <label class="inline-flex items-center">
                      <input
                        v-model="(selectedConnections[connection.socketId] as ManyConnectionState).mode"
                        type="radio"
                        value="individual"
                        class="mr-1"
                      />
                      Individual (create connection per entry)
                    </label>
                  </div>

                  <!-- Bulk mode: pick exactly one option to use for the entire array -->
                  <div
                    v-if="(selectedConnections[connection.socketId] as ManyConnectionState).mode === 'bulk'"
                  >
                    <p class="text-sm text-gray-500 mb-1">
                      Click a row below to select one match for the entire
                      array.
                    </p>
                    <div
                      class="grid grid-cols-3 gap-2 font-semibold border-b py-2"
                    >
                      <div>Component Name</div>
                      <div>Asset Name</div>
                      <div>Socket Name</div>
                    </div>
                    <div
                      v-for="match in bulkMatches(connection)"
                      :key="match.key"
                      :class="
                        clsx(
                          'grid grid-cols-3 p-2 border-b last:border-b-0 cursor-pointer',
                          selectedConnections[connection.socketId]?.socketId ===
                            match.key
                            ? ['bg-action-100 dark:bg-action-700']
                            : [
                                'dark:hover:text-action-300 hover:text-action-500',
                                'bg-neutral-100 dark:bg-neutral-700 group',
                              ],
                        )
                      "
                      @click="
                        selectSingleConnection(connection.socketId, match.key)
                      "
                    >
                      <div>{{ match.componentName }}</div>
                      <div>{{ match.schemaVariantName }}</div>
                      <div>
                        <TruncateWithTooltip>
                          {{ match.socketName }}
                        </TruncateWithTooltip>
                      </div>
                    </div>
                  </div>

                  <!-- Individual mode: pick a row for each entry in the array -->
                  <div v-else class="space-y-2">
                    <p class="text-sm text-gray-500 mb-1">
                      For each value, choose which connection should be made:
                    </p>

                    <div
                      v-for="(item, idx) in (connection.value as [])"
                      :key="idx"
                      class="p-2 bg-neutral-100 dark:bg-neutral-800"
                    >
                      <div class="mb-1 text-md font-semibold p-xs">
                        <TruncateWithTooltip>
                          "{{ item }}"
                        </TruncateWithTooltip>
                      </div>
                      <div
                        class="grid grid-cols-3 gap-2 font-semibold border-b py-2"
                      >
                        <div>Component Name</div>
                        <div>Asset Name</div>
                        <div>Socket Name</div>
                      </div>
                      <div
                        v-for="match in filteredMatches(connection, item)"
                        :key="match.key"
                        :class="clsx(
                        'grid grid-cols-3 p-2 border-b last:border-b-0 cursor-pointer',
                        (selectedConnections[connection.socketId] as ManyConnectionState).outputsPerIndex[idx] === match.key ? ['bg-action-100 dark:bg-action-700',] :
                          ['dark:hover:text-action-300 hover:text-action-500',
                            'bg-neutral-100 dark:bg-neutral-700 group'
                          ]
                      )"
                        @click="
                          selectIndividualOfMany(
                            connection.socketId,
                            idx,
                            match.key,
                          )
                        "
                      >
                        <div>{{ match.componentName }}</div>
                        <div>{{ match.schemaVariantName }}</div>
                        <div>
                          <TruncateWithTooltip>
                            {{ match.socketName }}
                          </TruncateWithTooltip>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </transition>
          </div>
        </div>
      </div>
      <!-- Footer button section -->
      <div class="flex justify-end mt-sm">
        <div class="flex flex-row gap-sm pt-sm">
          <VButton
            icon="x"
            label="Cancel"
            tone="shade"
            variant="ghost"
            @click="close"
          />
          <VButton
            class="flex-grow"
            icon="plus"
            label="Create Connection(s)"
            tone="action"
            :disabled="!canCreateAllConnections"
            @click="makeConnections"
          />
        </div>
      </div>
    </div>
  </Modal>
</template>

<script setup lang="ts">
import * as _ from "lodash-es";
import {
  Modal,
  useModal,
  VButton,
  TruncateWithTooltip,
  Icon,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { storeToRefs } from "pinia";
import {
  PotentialConnectionData,
  PotentialConnectionMatchData,
  useComponentsStore,
} from "@/store/components.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";

const componentsStore = useComponentsStore();
const featureFlagsStore = useFeatureFlagsStore();

const { autoconnectData } = storeToRefs(componentsStore);

const modalRef = ref<InstanceType<typeof Modal>>();
const { open: openModal, close: closeModal } = useModal(modalRef);

const errorMessages = ref<string[]>([]);

// Count how many were created
const createdConnections = computed(
  () => autoconnectData.value?.createdConnections ?? 0,
);

// For collapsible panels: track which socket IDs are expanded
const expandedConnections = ref<string[]>([]);
function toggleCollapse(key: string) {
  if (expandedConnections.value.includes(key)) {
    expandedConnections.value = expandedConnections.value.filter(
      (id) => id !== key,
    );
  } else {
    expandedConnections.value.push(key);
  }
}

// Storing state for user selections
type SingleConnectionState = {
  type: "single";
  socketId: string | null;
  state: "empty" | "hasSelections" | "satisfied";
};

type ManyConnectionState = {
  type: "many";
  mode: "bulk" | "individual";
  socketId: string | null;
  outputsPerIndex: (string | null)[];
  state: "empty" | "hasSelections" | "satisfied";
};

type ConnectionState = SingleConnectionState | ManyConnectionState;

const selectedConnections = ref<Record<string, ConnectionState>>({});

// Single-arity OR Many-arity + "bulk mode": user clicks a single row => store that selection
function selectSingleConnection(socketId: string, key: string) {
  const st = selectedConnections.value[socketId];
  if (st) {
    st.socketId = key;
    st.state = "satisfied";
  }
}

// Many-arity / "individual": user clicks a row for a given index
function selectIndividualOfMany(socketId: string, index: number, key: string) {
  const st = selectedConnections.value[socketId];
  if (st && st.type === "many") {
    st.outputsPerIndex[index] = key;
    if (st.outputsPerIndex.every((id) => id != null)) {
      st.state = "satisfied";
    } else if (st.outputsPerIndex.filter((id) => id != null).length > 0) {
      st.state = "hasSelections";
    }
  }
}
// Many-arity + "bulk mode": return only the matches that can handle the entire array's value.
// Ex: if "connection.value = [1,2,3]"
// then we only return matches whose "match.value" is also [1,2,3] (or [2,1,3] for that matter)
function bulkMatches(connection: PotentialConnectionData) {
  if (!Array.isArray(connection.value)) return [];
  return connection.processingConnections.filter((m) => {
    // Compare m.value to connection.value however you define "matches the entire array."
    // Simple example: arrays must be identical
    return arraysAreEqual(m.value, connection.value);
  });
}

// For "individual": "item" is the array entry - Return only matches whose value that match the item in question
// eslint-disable-next-line @typescript-eslint/no-explicit-any
function filteredMatches(connection: PotentialConnectionData, item: any) {
  return connection.processingConnections.filter((m) => {
    return m.value === item;
  });
}

// For "individual": check if each item in the array has at least one match
function canDoIndividual(connection: PotentialConnectionData) {
  if (!Array.isArray(connection.value) || !connection.value.length)
    return false;
  for (const item of connection.value) {
    // If no matches for this `item`, individual fails
    if (filteredMatches(connection, item).length === 0) return false;
  }
  return true;
}

// whether we're ready to hit the button or not!
const canCreateAllConnections = computed(() => {
  if (!autoconnectData.value) return false;

  const hasAnySelections = autoconnectData.value.potentialConnections.some(
    (connection) =>
      selectedConnections.value[connection.socketId]?.state === "hasSelections",
  );

  const hasAnySatisfied = autoconnectData.value.potentialConnections.some(
    (connection) =>
      selectedConnections.value[connection.socketId]?.state === "satisfied",
  );

  return !hasAnySelections && hasAnySatisfied;
});

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function arraysAreEqual(a: any[], b: any[]): boolean {
  if (a.length !== b.length) return false;
  return a.every((val) =>
    b.some((bVal) => JSON.stringify(val) === JSON.stringify(bVal)),
  );
}

// Loop through all selections and create those connections!
// TODO(brit): rethink how we make connections and expose potential errors. This isn't used yet.
const makeConnections = async () => {
  errorMessages.value = [];
  if (!autoconnectData.value) return;

  for (const connection of autoconnectData.value.potentialConnections) {
    if (selectedConnections.value[connection.socketId]?.state === "empty") {
      continue;
    }
    try {
      await connectThisSocket(connection);
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
    } catch (err: any) {
      errorMessages.value.push(
        `Error on socket ${connection.socketName}: ${err.message || err}`,
      );
    }
  }

  // If no errors, we can close the modal
  // TODO(brit): BUT WHAT IF THERE ARE ERRORS? A tomorrow problem for now
  if (!errorMessages.value.length) {
    close();
  }
};

// handles grabbing the selections to make connections for
async function connectThisSocket(connection: PotentialConnectionData) {
  const mainComponentId = autoconnectData.value?.componentId;
  if (!mainComponentId) return;
  const st = selectedConnections.value[connection.socketId];
  if (!st) return;
  if (connection.socketArity === "one") {
    // Single: find the chosen match
    const chosenKey = (st as SingleConnectionState).socketId;
    if (!chosenKey) return; // no selection?
    const chosenMatch = connection.processingConnections.find(
      (m) => m.key === chosenKey,
    );
    if (!chosenMatch)
      throw new Error(`Cannot find match for key: ${chosenKey}`);
    // Actually create the connection
    await doCreateConnection(connection, chosenMatch, mainComponentId);
  } else {
    // Many
    const manySt = st as ManyConnectionState;
    if (manySt.mode === "bulk") {
      const chosenKey = manySt.socketId;
      if (!chosenKey) return;
      const chosenMatch = connection.processingConnections.find(
        (m) => m.key === chosenKey,
      );
      if (!chosenMatch)
        throw new Error(`Cannot find bulk match for key: ${chosenKey}`);
      await doCreateConnection(connection, chosenMatch, mainComponentId);
    } else {
      // individual
      for (let i = 0; i < manySt.outputsPerIndex.length; i++) {
        const mk = manySt.outputsPerIndex[i];
        if (!mk) continue; // skip unselected
        const chosenMatch = connection.processingConnections.find(
          (m) => m.key === mk,
        );
        if (!chosenMatch)
          throw new Error(`Cannot find match for key: ${mk}, index ${i}`);
        await doCreateConnection(connection, chosenMatch, mainComponentId);
      }
    }
  }
}

// handles actually creating the connection based on direction
async function doCreateConnection(
  connection: PotentialConnectionData,
  potentialConnection: PotentialConnectionMatchData,
  mainComponentId: string,
) {
  // If direction is "input", "from" is potentialConnection; "to" is main component
  const to =
    connection.direction === "input"
      ? { componentId: mainComponentId, socketId: connection.socketId }
      : {
          componentId: potentialConnection.componentId,
          socketId: potentialConnection.socketId,
        };

  const from =
    connection.direction === "input"
      ? {
          componentId: potentialConnection.componentId,
          socketId: potentialConnection.socketId,
        }
      : { componentId: mainComponentId, socketId: connection.socketId };

  // Actually create the connection in the store
  const resp = await componentsStore.OVERRIDE_WITH_CONNECTION(
    from,
    to,
    connection.attributeValueId,
  );
  return resp;
}

function initializeSelections() {
  const map: Record<string, ConnectionState> = {};

  if (!autoconnectData.value) return;

  for (const pc of autoconnectData.value.potentialConnections) {
    if (pc.socketArity === "one") {
      // Single arity
      map[pc.socketId] = { type: "single", socketId: null, state: "empty" };
    } else {
      // Many arity
      const arrayLength = Array.isArray(pc.value) ? pc.value.length : 0;
      const isBulkCapable = bulkMatches(pc).length > 0;
      const isIndividualCapable = canDoIndividual(pc);

      let initialMode: "bulk" | "individual";
      if (isBulkCapable && isIndividualCapable) {
        // Both are possible => let's pick "bulk" by default
        initialMode = "bulk";
      } else if (isBulkCapable) {
        initialMode = "bulk";
      } else {
        // default to individual if bulk not possible
        initialMode = "individual";
      }

      map[pc.socketId] = {
        type: "many",
        mode: initialMode,
        socketId: null,
        outputsPerIndex: Array(arrayLength).fill(null),
        state: "empty",
      };
    }
  }

  selectedConnections.value = map;
}

const open = () => {
  // If feature flag is off or no potential connections, do nothing
  if (!featureFlagsStore.AUTOCONNECT) return;
  if (
    !autoconnectData.value ||
    autoconnectData.value.potentialConnections.length === 0
  )
    return;

  openModal();
  initializeSelections();
};

const close = () => {
  // clear state on close
  autoconnectData.value = null;
  selectedConnections.value = {};
  expandedConnections.value = [];
  closeModal();
};

// Event bus wiring for opening
const modelingEventBus = componentsStore.eventBus;
onMounted(() => {
  modelingEventBus.on("autoconnectComponent", open);
});
onBeforeUnmount(() => {
  modelingEventBus.off("autoconnectComponent", close);
});

defineExpose({ open, close });
</script>
