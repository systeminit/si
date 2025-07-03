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
import { useStatus } from "./logic_composables/status";
import { useRealtimeStore } from "@/store/realtime/realtime.store";
import { Context, assertIsDefined } from "@/newhotness/types";

const ctx = inject<Context>("CONTEXT");
assertIsDefined(ctx);

// This is needed for reacting to WsEvents.
const realtimeStore = useRealtimeStore();

const status = useStatus();
const bucket = reactive<Record<string, number>>({});
const trigger = ref<boolean>(false);

const timeoutMs = 30000;
const tickMs = 1000;

const key = useMakeKey();
const args = useMakeArgs();

const dependentValueComponentListQuery =
  useQuery<DependentValueComponentList | null>({
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
      // bucket.componentsInFlight = Date.now();
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
      // bucket.rainbow = Date.now();
    } else if (count === 0 && bucket.rainbow) {
      // BUCKET ITEM -- REMOVE -- RAINBOW, MATERIALIZED VIEWS, ETC.
      delete bucket.rainbow;
    }
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

  // TODO(nick): if we saw at least one failure, we should not say everything is synced.
  // if (deletedAtLeastOne) {
  // status.value = undefined;
  // }
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

const STATUS_PANEL_KEY = "statusPanel";
onMounted(() => {
  const interval = setInterval(() => {
    trigger.value = !trigger.value;
  }, tickMs);

  realtimeStore.subscribe(
    STATUS_PANEL_KEY,
    `changeset/${ctx?.changeSetId.value}`,
    [
      {
        eventType: "ManagementOperationsInProgress",
        callback: async (payload) => {
          // BUCKET ITEM -- ADD -- MANAGEMENT FUNCS
          const key = `management-${payload.requestUlid}`;
          bucket[key] = Date.now();
        },
      },
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
    ],
  );

  onUnmounted(() => {
    clearInterval(interval);
  });
});
onBeforeUnmount(() => {
  realtimeStore.unsubscribe(STATUS_PANEL_KEY);
});
</script>
