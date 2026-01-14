<template>
  <div
    :class="
      clsx(
        'group/pendingcard',
        'border rounded flex flex-row gap-xs items-center p-2xs cursor-pointer',
        themeClasses(
          'border-neutral-200 hover:border-action-500 hover:text-action-500 text-shade-100',
          'border-neutral-700 hover:border-action-300 hover:text-action-300 text-shade-0',
        ),
      )
    "
    @click="goToChangeSet(changeSet.id)"
  >
    <div class="group-hover/pendingcard:underline flex-1 min-w-0">
      <div class="font-bold line-clamp-2">
        {{ changeSet.name }}
      </div>
      <div
        :class="
          clsx(
            'text-xs italic',
            themeClasses(
              'text-neutral-500 group-hover/pendingcard:text-action-500',
              'text-neutral-400 group-hover/pendingcard:text-action-300',
            ),
          )
        "
      >
        <Timestamp :date="changeSet.mergeRequestedAt" showTimeIfToday size="extended" />

        by {{ changeSet.mergeRequestedByUser }}
      </div>
    </div>
    <div class="flex gap-xs flex-none">
      <NewButton
        label="Reject"
        tone="destructive"
        :loading="isRejecting"
        loadingText="Rejecting..."
        @click.stop="rejectChangeSet(changeSet.id)"
      />
      <NewButton
        tone="action"
        class="grow"
        label="Approve"
        :loading="isApproving"
        loadingText="Approving..."
        @click.stop="approveChangeSet(changeSet.id)"
      />
    </div>
  </div>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import * as _ from "lodash-es";
import { ref } from "vue";
import { themeClasses, Timestamp, NewButton } from "@si/vue-lib/design-system";
import { useRoute, useRouter } from "vue-router";
import { ChangeSet, ChangeSetId } from "@/api/sdf/dal/change_set";
import { useContext } from "../logic_composables/context";
import { useApi, routes, apiContextForChangeSet } from "../api_composables";

defineProps<{
  changeSet: ChangeSet;
}>();

const route = useRoute();
const router = useRouter();

const goToChangeSet = async (id: ChangeSetId) => {
  // Close the pending approval modal first
  emit("closeModal");

  // Navigate to the change set and add query parameter to trigger ApplyChangeSetModal
  const name = route.name;
  await router.push({
    name,
    params: {
      ...route.params,
      changeSetId: id,
    },
    query: {
      ...route.query,
      openApplyModal: "true",
    },
  });
};

const ctx = useContext();

// Track loading states locally
const isApproving = ref(false);
const isRejecting = ref(false);

const rejectChangeSet = async (id: ChangeSetId) => {
  isRejecting.value = true;
  try {
    const apiCtx = apiContextForChangeSet(ctx, id);
    const api = useApi(apiCtx);

    const call = api.endpoint(routes.ChangeSetApprove);
    const { req } = await call.post({ status: "Rejected" });
    if (api.ok(req)) {
      emit("closeModal");
    }
  } finally {
    isRejecting.value = false;
  }
};

const approveChangeSet = async (id: ChangeSetId) => {
  isApproving.value = true;
  try {
    const apiCtx = apiContextForChangeSet(ctx, id);
    const api = useApi(apiCtx);

    const call = api.endpoint(routes.ChangeSetApprove);
    const { req } = await call.post({ status: "Approved" });
    if (api.ok(req)) {
      emit("closeModal");
    }
  } finally {
    isApproving.value = false;
  }
};

const emit = defineEmits<{
  (e: "closeModal"): void;
}>();
</script>
