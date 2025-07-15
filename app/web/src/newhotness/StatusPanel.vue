<template>
  <div class="mt-xs ml-xs relative">
    <StatusPanelIcon :status="status" @faded="() => (status = undefined)" />
  </div>
</template>

<script lang="ts" setup>
import {
  computed,
  ref,
  reactive,
  watch,
  onMounted,
  onUnmounted,
  unref,
  inject,
  onBeforeUnmount,
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
import { Context, assertIsDefined } from "@/newhotness/types";
import { useStatus } from "./logic_composables/status";

const realtimeStore = useRealtimeStore();
const ctx = inject<Context>("CONTEXT");
assertIsDefined(ctx);

const status = useStatus();
const bucket = reactive<Record<string, number>>({});
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
      bucket.componentsInFlight = Date.now();
    } else if (
      newComponentsInFlight.length === 0 &&
      bucket.componentsInFlight
    ) {
      // BUCKET ITEM -- REMOVE -- COMPONENTS IN FLIGHT, DVU ROOTS, ETC.
      delete bucket.componentsInFlight;
    }
  },
);

const rainbow = useRainbow();
watch(
  () => rainbow.value,
  (newRainbow) => {
    const count = unref(newRainbow.count);
    if (count > 0) {
      // BUCKET ITEM -- ADD -- RAINBOW, MATERIALIZED VIEWS, ETC.
      bucket.rainbow = Date.now();
    } else if (count === 0 && bucket.rainbow) {
      // BUCKET ITEM -- REMOVE -- RAINBOW, MATERIALIZED VIEWS, ETC.
      delete bucket.rainbow;
    }
  },
);

const STATUS_PANEL_KEY = "statusPanel";
const changeSetId = computed(() => ctx.changeSetId.value);
watch(
  () => changeSetId.value,
  () => {
    realtimeStore.unsubscribe(STATUS_PANEL_KEY);
    realtimeStore.subscribe(
      STATUS_PANEL_KEY,
      `changeset/${changeSetId.value}`,
      [
        {
          eventType: "ManagementOperationsComplete",
          callback: async (payload) => {
            if (!payload.requestUlid) return;
            const key = `management-${payload.requestUlid}`;
            if (bucket[key]) {
              // BUCKET ITEM -- REMOVE -- MANAGEMENT FUNCS
              delete bucket[key];
            }
          },
        },
        {
          eventType: "ManagementOperationsFailed",
          callback: async (payload) => {
            // BUCKET ITEM -- ADD -- MANAGEMENT FUNCS
            const key = `management-${payload.requestUlid}`;
            bucket[key] = Date.now();
          },
        },
        {
          eventType: "ManagementOperationsInProgress",
          callback: async (payload) => {
            // BUCKET ITEM -- ADD -- MANAGEMENT FUNCS
            const key = `management-${payload.requestUlid}`;
            bucket[key] = Date.now();
          },
        },
      ],
    );
  },
);

// This watcher ejects expired items.
watch([trigger], () => {
  const now = Date.now();

  for (const [key, value] of Object.entries(bucket)) {
    if (now - value > timeoutMs) {
      delete bucket[key];
    }
  }
});

// This watcher updates the status based on the state of the bucket.
watch(
  () => ({ ...bucket }),
  (newBucket) => {
    if (Object.keys(newBucket).length < 1) {
      status.value = "synced";
    } else {
      status.value = "syncing";
    }
  },
);

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
