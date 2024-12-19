<template>
  <VButton
    v-if="
      featureFlagsStore.DEV_SLICE_REBASING && statusWithBase?.conflictsWithBase
    "
    size="md"
    tone="destructive"
    disabled
    square
    label="Cannot Merge or Rebase, Resolve Conflicts First"
  >
  </VButton>
  <VButton
    v-else-if="
      featureFlagsStore.DEV_SLICE_REBASING && statusWithBase?.baseHasUpdates
    "
    size="md"
    tone="warning"
    square
    label="Rebase from Head"
    @click="rebase"
  >
  </VButton>
  <!-- TODO: we can change this v-else-if to look at `statusWithBase.changeSetHasUpdates` -->
  <VButton
    v-else-if="!changeSetsStore.headSelected"
    ref="applyButtonRef"
    size="md"
    tone="success"
    loadingText="Applying Changes"
    :requestStatus="applyChangeSetReqStatus"
    :disabled="statusStoreUpdating"
    square
    @click.stop="openApprovalFlowModal"
  >
    <div class="px-xs dark:text-neutral-800 font-medium">Apply Change Set</div>

    <template #icon>
      <Icon name="tools" class="dark:text-neutral-800" />
    </template>

    <template #iconRight>
      <PillCounter
        :count="displayCount"
        :paddingX="displayCount > 10 ? '2xs' : 'xs'"
        noColorStyles
        :class="
          clsx(
            'text-xl font-bold',
            themeClasses(
              'bg-success-600 text-shade-0',
              'bg-success-300 text-success-900',
            ),
          )
        "
      />
    </template>

    <ApprovalFlowModal
      v-if="!featureFlagsStore.REBAC"
      ref="approvalFlowModalRef"
      votingKind="merge"
      @completeVoting="applyChangeSet"
    />

    <ApprovalFlowModal2 v-else ref="approvalFlowModal2Ref" votingKind="merge" />
  </VButton>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import * as _ from "lodash-es";
import { useRouter, useRoute } from "vue-router";
import {
  VButton,
  Icon,
  PillCounter,
  themeClasses,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { useToast } from "vue-toastification";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useStatusStore } from "@/store/status.store";
import { useActionsStore } from "@/store/actions.store";
import { useAuthStore } from "@/store/auth.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import RetryApply from "@/components/toasts/RetryApply.vue";
import { useViewsStore } from "@/store/views.store";
import ApprovalFlowModal from "./ApprovalFlowModal.vue";
import ApprovalFlowModal2 from "./ApprovalFlowModal2.vue";

const featureFlagsStore = useFeatureFlagsStore();
const actionsStore = useActionsStore();
const authStore = useAuthStore();
const changeSetsStore = useChangeSetsStore();
const route = useRoute();
const router = useRouter();
const statusStore = useStatusStore();
const toast = useToast();

const displayCount = computed(() => actionsStore.proposedActions.length);

const applyChangeSetReqStatus =
  changeSetsStore.getRequestStatus("APPLY_CHANGE_SET");

const approvalFlowModalRef = ref<InstanceType<typeof ApprovalFlowModal> | null>(
  null,
);
const approvalFlowModal2Ref = ref<InstanceType<
  typeof ApprovalFlowModal2
> | null>(null);

const openApprovalFlowModal = () => {
  if (featureFlagsStore.REBAC) {
    approvalFlowModal2Ref.value?.open();
  } else {
    approvalFlowModalRef.value?.open();
  }
};

// Applies the current change set3
const applyChangeSet = async () => {
  if (!route.name) return;
  // if (featureFlagsStore.REBAC) return;
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  const viewsStore = useViewsStore(changeSetsStore.headChangeSetId!);
  // need to clear selections prior to applying, having them is causing bugs (BUG-725)
  viewsStore.clearSelections();
  viewsStore.syncSelectionIntoUrl();
  const resp = await changeSetsStore.APPLY_CHANGE_SET(
    authStore.user?.email ?? "",
  );
  if (resp.result.success) {
    router.replace({
      name: route.name,
      params: {
        ...route.params,
        changeSetId: "head",
      },
    });
  } else if (resp.result.statusCode === 428) {
    toast({
      component: RetryApply,
    });
  }
};

const statusStoreUpdating = computed(() => {
  if (statusStore.globalStatus) {
    return statusStore.globalStatus.isUpdating;
  } else return false;
});

const statusWithBase = computed(
  () =>
    changeSetsStore.statusWithBase[changeSetsStore.selectedChangeSetId || ""],
);

const rebase = () => {
  if (changeSetsStore.selectedChangeSetId)
    changeSetsStore.REBASE_ON_BASE(changeSetsStore.selectedChangeSetId);
};
</script>
