<template>
  <StatusBarTab
    :class="clsx(numConflicts > 0 ? 'animate-bounce cursor-pointer' : '')"
    :selected="props.selected"
    :click="() => numConflicts > 0 && openModal()"
  >
    <template #icon>
      <Icon class="text-destructive-600" name="read-only" />
    </template>
    <template #name>Conflicts</template>
    <template #summary>
      <StatusBarTabPill
        class="bg-destructive-100 text-destructive-600 font-bold"
      >
        <div @click="() => numConflicts > 0 && openModal()">
          {{ numConflicts }}
        </div>
      </StatusBarTabPill>
      <Modal ref="modalRef" title="Conflicts" noExit>
        <div>
          <div class="flex flex-row gap-sm items-center">
            <Icon
              name="alert-circle"
              class="text-warning-600 content-center ml-md"
              size="lg"
            />
            <p v-if="numConflicts > 0" class="grow py-md">
              Your changes have produced
              <strong class="underline">{{ numConflicts }} conflict(s)</strong>.
              These changes have not been applied. If necessary, retry them
              below:
            </p>
            <p v-else>No more conflicts!</p>
          </div>
          <ol
            v-if="numConflicts > 0"
            class="max-h-[50vh] overflow-hidden overflow-y-auto"
          >
            <li
              v-for="conflict in conflictDisplay"
              :key="conflict.requestUlid"
              class="text-sm py-sm border-b-2 border-neutral-200 dark:border-neutral-600 flex flex-col"
            >
              <span class="capitalize self-center"
                >{{ conflict.actionName }}:</span
              >
              <pre class="text-xs">{{ conflict.payload }}</pre>
              <VButton
                class="self-end"
                label="Retry Change"
                size="md"
                tone="action"
                @click="retry(conflict.requestUlid)"
              ></VButton>
            </li>
          </ol>
          <div class="flex flex-row gap-sm items-center mt-sm">
            <VButton
              class="grow text-action-300 dark:hover:text-white hover:text-black hover:underline"
              label="Close"
              tone="empty"
              variant="solid"
              @click="modalRef.close()"
            ></VButton>
          </div>
        </div>
      </Modal>
    </template>
  </StatusBarTab>
</template>

<script setup lang="ts">
import { ref, computed, reactive, watch, WatchStopHandle } from "vue";
import { Store } from "pinia";
import clsx from "clsx";
import { RequestUlid, ConflictsForRetry, ApiRequest } from "@si/vue-lib/pinia";
import { Icon, Modal, VButton } from "@si/vue-lib/design-system";
import { useComponentAttributesStore } from "@/store/component_attributes.store";
import { useActionsStore } from "@/store/actions.store";
import { useAssetStore } from "@/store/asset.store";
import { useComponentsStore } from "@/store/components.store";
import StatusBarTabPill from "./StatusBarTabPill.vue";
import StatusBarTab from "./StatusBarTab.vue";

const modalRef = ref();

const openModal = () => {
  modalRef.value.open();
};

const props = defineProps({
  selected: Boolean,
});

const assetStore = useAssetStore();
const actionsStore = useActionsStore();
const componentsStore = useComponentsStore();

let attributesStore: Store | null;
const conflicts = reactive({} as ConflictsForRetry);
let unwatch: WatchStopHandle | undefined;

watch(
  () => componentsStore.selectedComponentId,
  () => {
    if (componentsStore.selectedComponentId) {
      attributesStore = useComponentAttributesStore(
        componentsStore.selectedComponentId,
      );
      if (unwatch) unwatch();
      unwatch = watch(attributesStore.availableRetries, () => {
        Object.assign(conflicts, attributesStore?.availableRetries);
      });
    } else {
      attributesStore = null;
      if (unwatch) unwatch();
    }
  },
  { immediate: true },
);

watch(componentsStore.availableRetries, () =>
  Object.assign(conflicts, componentsStore.availableRetries),
);
watch(assetStore.availableRetries, () =>
  Object.assign(conflicts, assetStore.availableRetries),
);
watch(actionsStore.availableRetries, () =>
  Object.assign(conflicts, actionsStore.availableRetries),
);

const numConflicts = computed(() => Object.keys(conflicts).length);

type ConflictDisplay = {
  requestUlid: RequestUlid;
  actionName: string;
  payload: Record<string, unknown> | undefined;
};

const retry = (requestUlid: RequestUlid) => {
  let p;
  if (Object.keys(componentsStore.availableRetries).includes(requestUlid))
    p = componentsStore.RETRY_CONFLICT(requestUlid);
  else if (Object.keys(assetStore.availableRetries).includes(requestUlid))
    p = assetStore.RETRY_CONFLICT(requestUlid);
  else if (Object.keys(actionsStore.availableRetries).includes(requestUlid))
    p = actionsStore.RETRY_CONFLICT(requestUlid);
  else if (
    attributesStore &&
    Object.keys(attributesStore.availableRetries).includes(requestUlid)
  )
    p = attributesStore.RETRY_CONFLICT(requestUlid);

  if (p) delete conflicts[requestUlid];
  else throw Error("Retry not found");
};

const conflictDisplay = computed(() => {
  const display = [] as ConflictDisplay[];
  for (const requestUlid of Object.keys(conflicts)) {
    const details = conflicts[requestUlid];
    const actionName =
      details?.[0].replaceAll("_", " ").toLowerCase() || "Unknown";
    const apiRequest = details?.[1] as ApiRequest;
    const payload = apiRequest?.requestSpec.params;
    display.push({
      requestUlid,
      actionName,
      payload,
    });
  }
  return display;
});
</script>

<style type="less">
pre {
  white-space: pre-wrap; /* Since CSS 2.1 */
}
</style>
