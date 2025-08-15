<template>
  <div class="mt-xs ml-xs relative">
    <StatusPanelIcon
      :status="status[changeSetId]"
      @faded="() => delete status[changeSetId]"
    />
  </div>
</template>

<script lang="ts" setup>
import {
  computed,
  ref,
  watch,
  onMounted,
  onUnmounted,
  unref,
  onBeforeUnmount,
  reactive,
} from "vue";
import { useQuery } from "@tanstack/vue-query";
import StatusPanelIcon from "@/newhotness/StatusPanelIcon.vue";
import { useMakeKey, bifrost, useMakeArgs } from "@/store/realtime/heimdall";
import {
  DependentValueComponentList,
  EntityKind,
} from "@/workers/types/entity_kind_types";
import { useRainbow } from "@/newhotness/logic_composables/rainbow_counter";
import { useRealtimeStore } from "@/store/realtime/realtime.store";
import { ChangeSetId, ChangeSetStatus } from "@/api/sdf/dal/change_set";
import { useStatus } from "./logic_composables/status";
import { useContext } from "./logic_composables/context";

const realtimeStore = useRealtimeStore();

const ctx = useContext();
const changeSetId = computed(() => ctx.changeSetId.value);

export type PerChangeSet = Record<string, number>;
const superBucket = reactive<Record<ChangeSetId, PerChangeSet>>({});

const status = useStatus();
const bucketIsEmpty = computed(() => {
  const k = Object.keys(superBucket[changeSetId.value] ?? {});
  return k.length < 1;
});

const trigger = ref<boolean>(false);

const timeoutMs = 30000;
const tickMs = 1000;

const key = useMakeKey();
const args = useMakeArgs();

const dependentValueComponentListQuery =
  useQuery<DependentValueComponentList | null>({
    enabled: ctx.queriesEnabled,
    queryKey: key(EntityKind.DependentValueComponentList),
    queryFn: async () =>
      await bifrost<DependentValueComponentList>(
        args(EntityKind.DependentValueComponentList),
      ),
  });
const componentsInFlight = computed(
  () => dependentValueComponentListQuery.data.value?.componentIds ?? [],
);
watch(
  () => componentsInFlight.value,
  (newComponentsInFlight) => {
    if (newComponentsInFlight.length > 0) {
      // BUCKET ITEM -- ADD -- COMPONENTS IN FLIGHT, DVU ROOTS, ETC.
      let bucket = superBucket[changeSetId.value];
      if (!bucket) {
        bucket = {};
        superBucket[changeSetId.value] = bucket;
      }
      bucket.componentsInFlight = Date.now();
    } else if (
      newComponentsInFlight.length === 0 &&
      superBucket[changeSetId.value]?.componentsInFlight
    ) {
      // BUCKET ITEM -- REMOVE -- COMPONENTS IN FLIGHT, DVU ROOTS, ETC.
      delete superBucket[changeSetId.value]?.componentsInFlight;
    }
  },
);

const rainbow = useRainbow(changeSetId);
watch(
  () => rainbow.value,
  (newRainbow) => {
    const count = unref(newRainbow.count);
    if (count > 0) {
      // BUCKET ITEM -- ADD -- RAINBOW, MATERIALIZED VIEWS, ETC.
      let bucket = superBucket[changeSetId.value];
      if (!bucket) {
        bucket = {};
        superBucket[changeSetId.value] = bucket;
      }
      bucket.rainbow = Date.now();
    } else if (count === 0 && superBucket[changeSetId.value]?.rainbow) {
      // BUCKET ITEM -- REMOVE -- RAINBOW, MATERIALIZED VIEWS, ETC.
      delete superBucket[changeSetId.value]?.rainbow;
    }
  },
  { deep: true },
);

const STATUS_PANEL_KEY = "statusPanel";
realtimeStore.subscribe(
  STATUS_PANEL_KEY,
  `workspace/${ctx.workspacePk.value}`,
  [
    {
      eventType: "ChangeSetStatusChanged",
      callback: async (data) => {
        if (
          [
            ChangeSetStatus.Abandoned,
            ChangeSetStatus.Applied,
            ChangeSetStatus.Closed,
          ].includes(data.changeSet.status) &&
          data.changeSet.id !== ctx.headChangeSetId.value
        ) {
          if (status[data.changeSet.id]) {
            delete status[data.changeSet.id];
          }
          delete superBucket[data.changeSet.id];
        }
      },
    },
    {
      eventType: "ManagementOperationsComplete",
      callback: async (payload, meta) => {
        if (!payload.requestUlid) return;
        const key = `management-${payload.requestUlid}`;
        if (superBucket[meta.change_set_id]?.[key]) {
          // BUCKET ITEM -- REMOVE -- MANAGEMENT FUNCS
          delete superBucket[meta.change_set_id]?.[key];
        }
      },
    },
    {
      eventType: "ManagementOperationsFailed",
      callback: async (payload, meta) => {
        // BUCKET ITEM -- ADD -- MANAGEMENT FUNCS
        const key = `management-${payload.requestUlid}`;
        let bucket = superBucket[meta.change_set_id];
        if (!bucket) {
          bucket = {};
          superBucket[meta.change_set_id] = bucket;
        }
        bucket[key] = Date.now();
      },
    },
    {
      eventType: "ManagementOperationsInProgress",
      callback: async (payload, meta) => {
        // BUCKET ITEM -- ADD -- MANAGEMENT FUNCS
        const key = `management-${payload.requestUlid}`;
        let bucket = superBucket[meta.change_set_id];
        if (!bucket) {
          bucket = {};
          superBucket[meta.change_set_id] = bucket;
        }
        bucket[key] = Date.now();
      },
    },
  ],
);

// This watcher ejects expired items.
watch([trigger, changeSetId.value], () => {
  const now = Date.now();

  for (const [key, value] of Object.entries(
    superBucket[changeSetId.value] ?? {},
  )) {
    if (now - value > timeoutMs) {
      delete superBucket[changeSetId.value]?.[key];
    }
  }
});

// This watcher updates the status based on the state of the bucket.
watch(bucketIsEmpty, (newBucketIsEmpty) => {
  if (newBucketIsEmpty) {
    status[changeSetId.value] = "synced";
  } else {
    status[changeSetId.value] = "syncing";
  }
});

onMounted(() => {
  const interval = setInterval(() => {
    trigger.value = !trigger.value;
  }, tickMs);

  onUnmounted(() => {
    clearInterval(interval);
  });
});
onBeforeUnmount(() => {
  realtimeStore.unsubscribe(STATUS_PANEL_KEY);
});
</script>
